use std::{
    fmt::Debug,
    io::{Seek, SeekFrom, Write},
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    body::Bytes,
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};

use crate::{
    AppState,
    db::{
        file::{File, FileAccess},
        links::FileLink,
    },
    simply_packet::{ByteConversion, JsonChunkIndex, JsonData, Packet, packet},
    upload::path_is_valid,
};

pub struct WebsocketData {
    pub state: Arc<AppState>,
    pub addr: SocketAddr,
    pub id: String,
    pub path: String,
    pub link: Option<FileLink>,
}

impl Debug for WebsocketData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WebSocketData: {}, {}, {}",
            self.id, self.addr, self.path
        )
    }
}

pub async fn upload_handler(ws: WebSocketUpgrade, data: WebsocketData) -> Response {
    ws.max_frame_size(64_000_000)
        .on_upgrade(move |socket| handle_socket(socket, data))
}

async fn handle_socket(mut socket: WebSocket, mut data: WebsocketData) {
    tracing::trace!("New websocket connection: {:?}", data);

    // we put this in most upper scope so no matter what we can save the chunk_index
    let mut chunk_index: u64 = 0;
    match async {
        socket
            .send(message!(JsonData::ConnectionAccepted))
            .await
            .map_err(|e| UploadError::FailedToSend(e))?;
        tracing::trace!("Sent ConnectionAccepted [Packet#1]");

        let file = if let Some(msg) = socket.recv().await {
            if let Ok(Message::Binary(data)) = msg {
                let packet = Packet::from_bytes(&data).map_err(|e| UploadError::PacketError(e))?;
                // ^ this packet must be a InitializeUpload
                if let Packet::Json(JsonData::InitializeUpload(file)) = packet {
                    tracing::trace!("Received InitializeUpload: {:?} [Packet#2]", file);
                    file
                } else {
                    return Err(UploadError::InvalidPacketOrder);
                }
            } else {
                tracing::error!("{msg:#?}");
                return Err(UploadError::InvalidMessageType);
            }
        } else {
            return Err(UploadError::ClientDisconnected);
        };

        tracing::trace!("Preparing for upload, creating file handler");
        let total_chunks = (file.size as f64 / file.chunk_size as f64).ceil() as u64;

        if !path_is_valid(&data.path) {
            tracing::error!("{:?} is invalid", data.path);
            return Err(UploadError::InvalidPath(data.path));
        }

        let bytes_stored = File::get_bytes_stored(&data.state.db).await.unwrap();
        if bytes_stored > data.state.config.storage_limit as u64 {
            return Err(UploadError::InsufficientStorage);
        }

        let exists_in_db = match File::get_via_path(&data.state.db, &data.path).await {
            Ok(f) => {
                // if it already exists, copy it id for later
                // mostly for the last err in the most outer scope
                // that tries to save the chunk_index
                data.id = f.id.clone();
                // if the total_chunks are mismatched we probably got a new file
                // and thus ""must"" discard the old one and begin from the start.
                // so we clear the file in db and on disk & prepare it
                // to make a new entry etc with the new data
                if total_chunks as i64 != f.total_chunks {
                    tracing::error!(
                        "Mismatched total_chunks, uploading file({}) from the start",
                        &data.id
                    );

                    File::delete(&data.state.db, &data.id)
                        .await
                        .map_err(|e| UploadError::DBError(e))?;

                    if data
                        .state
                        .fs
                        .exists(&data.path)
                        .await
                        .map_err(|e| UploadError::FailedIO(e))?
                    {
                        data.state
                            .fs
                            .delete(&data.path)
                            .await
                            .map_err(|e| UploadError::FailedIO(e))?;
                    }

                    None
                } else {
                    Some(f)
                }
            }
            Err(sqlx::Error::RowNotFound) => None,
            Err(err) => return Err(UploadError::DBError(err)),
        };

        // if theres no entry already from above, we do create a new one
        // this is so we can "resume" uploads from existing database entires
        // and its latest chunk_index
        let mut db_file = match exists_in_db {
            Some(f) => f,
            None => File::new(&data.state.db, &data.id, &data.path, total_chunks as i64)
                .await
                .map_err(|e| UploadError::DBError(e))?,
        };

        let file_handler = data
            .state
            .fs
            .get_file_handler(&data.path)
            .await
            .map_err(|e| UploadError::FailedIO(e))?;
        let mut writer = std::io::BufWriter::new(file_handler);
        chunk_index = db_file.chunk_index as u64; // start at whatever chunk_index is from db, if new it defaults to 0

        tracing::trace!(
            "Chunk metdata: total: {}, bytes per: {}, starting_index: {}",
            total_chunks,
            file.chunk_size,
            chunk_index
        );

        socket
            .send(message!(JsonData::ReadyForUpload(JsonChunkIndex {
                chunk_index
            })))
            .await
            .map_err(|e| UploadError::FailedToSend(e))?;
        tracing::trace!("Sent ReadyForUpload [Packet#3]");

        let (mut sender, mut rec) = socket.split();

        let upload_result: Result<(), UploadError> = {
            while let Some(msg) = rec.next().await {
                if let Ok(msg) = msg {
                    if let Message::Binary(msg_data) = msg {
                        let packet = Packet::from_bytes(&msg_data)
                            .map_err(|e| UploadError::PacketError(e))?;

                        match packet {
                            Packet::Binary(chunk) => {
                                if chunk_index != chunk.idx as u64 {
                                    tracing::error!(
                                        "Out of order chunks, server: {}, client: {}. Backtracking",
                                        chunk_index,
                                        chunk.idx
                                    );
                                    sender
                                        .send(message!(JsonData::SetChunkIndex(JsonChunkIndex {
                                            chunk_index
                                        })))
                                        .await
                                        .map_err(|e| UploadError::FailedToSend(e))?;
                                }

                                writer
                                    .seek(SeekFrom::Start(chunk_index * file.chunk_size))
                                    .map_err(|e| UploadError::FailedIO(e))?;
                                writer
                                    .write_all(chunk.data)
                                    .map_err(|e| UploadError::FailedIO(e))?;

                                chunk_index += 1;
                                if chunk_index % 1000 == 0 {
                                    tracing::debug!(
                                        "[{}] Got 1000nth chunk: {chunk_index}/{total_chunks}",
                                        &data.id
                                    );
                                }

                                if chunk_index >= total_chunks {
                                    tracing::trace!(
                                        "File upload completed, received all {total_chunks} chunks"
                                    );
                                    break;
                                }

                                // send next ack
                                sender
                                    .send(Message::Binary(Bytes::from(
                                        Packet::Next
                                            .to_bytes()
                                            .map_err(|e| UploadError::PacketError(e))?,
                                    )))
                                    .await
                                    .map_err(|e| UploadError::FailedToSend(e))?;
                            }
                            _ => {
                                return Err(UploadError::UnexpectedPacketType);
                            }
                        }
                    } else if let Message::Close(Some(close)) = msg {
                        tracing::trace!(
                            "websocket close message: {}, {}",
                            close.code,
                            close.reason.to_string()
                        );
                        return Err(UploadError::ClientDisconnected);
                    } else {
                        return Err(UploadError::UnexpectedMessageType);
                    }
                } else if let Err(err) = msg {
                    return Err(UploadError::MessageIsNotOk(err));
                }
            }

            Ok(())
        };
        // if the core upload fails or succeds it will always run code here
        writer.flush().map_err(|e| UploadError::FailedIO(e))?;
        // always, even if it fails or not. update the databases chunk index
        // this is so we can resume uploading AND this code is 100%
        // always gonna run even if the chunked upload part fails or not
        db_file
            .update_chunk_index(&data.state.db, chunk_index as i64)
            .await
            .map_err(|e| UploadError::DBError(e))?;

        match upload_result {
            Ok(_) => {
                db_file
                    .successful_upload(&data.state.db, file.size as i64)
                    .await
                    .unwrap();

                // one-time link handling
                if let Some(link) = data.link {
                    link.uploaded_with(&data.state.db, &data.id)
                        .await
                        .map_err(|e| UploadError::DBError(e))?;
                    // // always change one-time "public" uploads to well, Public
                    db_file
                        .change_access(&data.state.db, FileAccess::Public)
                        .await
                        .map_err(|e| UploadError::DBError(e))?;
                }

                // We do send the entire DB file BUT
                // for link uploads its always .public_uploads anyway
                // and theres no sensetive data
                sender
                    .send(message!(JsonData::UploadComplete(db_file)))
                    .await
                    .map_err(|e| UploadError::FailedToSend(e))?;

                tracing::trace!("Sent UploadComplete [Packet#5]");
            }
            Err(err) => {
                // try and save chunk_index at last moment
                File::get_via_id(&data.state.db, &data.id)
                    .await
                    .map_err(|e| UploadError::DBError(e))?
                    .update_chunk_index(&data.state.db, chunk_index as i64)
                    .await
                    .map_err(|e| UploadError::DBError(e))?;

                // propogate it
                return Err(err);
            }
        }

        Ok::<(), UploadError>(())
    }
    .await
    {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("{err:?}");

            // always try and save that chunk index
            match File::get_via_id(&data.state.db, &data.id).await {
                Ok(mut f) => {
                    match f
                        .update_chunk_index(&data.state.db, chunk_index as i64)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => tracing::error!("{e:?}"),
                    }
                }
                Err(e) => tracing::error!("{e:?}"),
            };
            return;
        }
    }

    tracing::trace!("Closing websocket connection: {:?}", data.addr.ip());
}

macro_rules! message {
    ($($input:tt)*) => {{
        use axum::{extract::ws::{Message}, body::Bytes};
        Message::Binary(Bytes::from(packet!($($input)*).map_err(|e| UploadError::PacketError(e))?))
    }};
}
pub(crate) use message;

#[derive(Debug)]
#[allow(unused)]
enum UploadError {
    UnexpectedMessageType,
    UnexpectedPacketType,
    InvalidPacketOrder,
    InvalidMessageType,
    ClientDisconnected,
    InsufficientStorage,
    InvalidPath(String),
    MessageIsNotOk(axum::Error),
    FailedToSend(axum::Error),
    FailedIO(std::io::Error),
    PacketError(crate::simply_packet::PacketError),
    DBError(sqlx::Error),
}

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UploadError {:?}", self)
    }
}
impl std::error::Error for UploadError {}
