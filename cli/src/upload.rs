use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom, Write, stdout},
    path::PathBuf,
    time::Instant,
};

use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use human_bytes::human_bytes;
use sf_core::{
    FileAccess,
    simply_packet::{ByteConversion, Chunk, JsonData, JsonInitializeUpload, Packet},
};
use tungstenite::{
    Bytes, ClientRequestBuilder, Message, client::connect_with_config, protocol::WebSocketConfig,
};

use crate::{
    app::App,
    args::{ArgPath, FileIdentifier},
};

const CHUNK_SIZE: u64 = 16 * 1024 * 1024;

pub fn upload(
    app: App,
    local: PathBuf,
    remote: ArgPath,
    access: Option<FileAccess>,
    id: Option<String>,
) {
    let base_socket_url = app.get_base_socket_url();
    let socket_url = base_socket_url + if id.is_some() { "/o/" } else { "/m/" } + "upload/";

    let remote = remote.path();
    let path = remote
        .join(local.file_name().unwrap())
        .to_string_lossy()
        .to_string();

    let socket_url = (socket_url + &path).replace("\\", "/");
    let host = app.get_host();
    #[allow(unused_assignments)]
    let mut chunk_index: u64 = 0;
    let total_chunks = (local.metadata().unwrap().len() as f64 / CHUNK_SIZE as f64).ceil() as i64;

    tracing::debug!("Connect to websocket: {socket_url}");

    let mut req = ClientRequestBuilder::new(socket_url.parse().unwrap());
    if let Some(token) = host.token {
        req = req.with_header("Authorization", format!("Bearer {token}"));
    }

    let (mut socket, response) = connect_with_config(
        req,
        Some(
            // I think one of these need to be upped to allow CHUNK_SIZE
            // but im just cranking them all up to be sure and i mean, it works™
            WebSocketConfig::default()
                .max_frame_size(Some((CHUNK_SIZE * 2) as usize))
                .max_message_size(Some((CHUNK_SIZE * 2) as usize))
                .read_buffer_size((CHUNK_SIZE * 2) as usize)
                .write_buffer_size((CHUNK_SIZE * 2) as usize),
        ),
        1,
    )
    .expect("Failed to connect to websocket");

    if response.status().as_u16() != 101 {
        return tracing::error!("Failed to switch protocols");
    }

    let conn_accepted = socket.read().unwrap();
    if let Message::Binary(bin) = conn_accepted {
        let packet = Packet::from_bytes(&bin).unwrap();
        match packet {
            Packet::Json(JsonData::ConnectionAccepted) => tracing::debug!("ConnectionAccepted"),
            _ => return tracing::error!("Got invalid packet message"),
        }
    } else {
        return tracing::error!("Got invalid Message type");
    }

    let init_data = JsonInitializeUpload {
        name: local.file_name().unwrap().to_string_lossy().to_string(),
        size: local.metadata().unwrap().len(),
        chunk_size: CHUNK_SIZE,
    };

    socket
        .send(Message::Binary(Bytes::from(
            Packet::Json(JsonData::InitializeUpload(init_data))
                .to_bytes()
                .unwrap(),
        )))
        .unwrap();
    tracing::debug!("Sent InitializeUpload");

    let ready_upload = socket.read().unwrap();
    if let Message::Binary(bin) = ready_upload {
        let packet = Packet::from_bytes(&bin).unwrap();
        match packet {
            Packet::Json(JsonData::ReadyForUpload(data)) => chunk_index = data.chunk_index,
            _ => return tracing::error!("Got invalid packet message"),
        }
    } else {
        return tracing::error!("Got invalid Message type");
    }

    let file = File::open(&local).unwrap();
    tracing::debug!(
        "Got ReadyForUpload, going into upload loop ({} bytes, {} idx)",
        file.metadata().unwrap().len(),
        chunk_index
    );
    tracing::info!("Starting upload on chunk {chunk_index}/{total_chunks}");
    let mut file = BufReader::new(file);

    let mut stdout = stdout();
    let start_time = Instant::now();
    let mut bytes_sent = 0;
    stdout.execute(cursor::Hide).unwrap();

    loop {
        stdout.queue(cursor::SavePosition).unwrap();

        file.seek(SeekFrom::Start(chunk_index * CHUNK_SIZE))
            .unwrap();
        let mut handler = (&mut file).take(CHUNK_SIZE);

        let mut chunk_data = vec![0; CHUNK_SIZE as usize];
        let bytes = handler.read(&mut chunk_data).unwrap();

        if bytes == 0 {
            return tracing::error!("chunk len is 0");
        }

        let chunk = Chunk {
            idx: chunk_index,
            size: bytes as u64,
            data: &chunk_data[0..bytes], // we only want the exact chunk so we trim it based off what we read
        };

        socket
            .send(Message::Binary(Bytes::from(
                Packet::Binary(chunk).to_bytes().unwrap(),
            )))
            .unwrap();
        bytes_sent += bytes;

        let upload_speed = bytes_sent as f64 / (start_time.elapsed().as_millis() as f64 / 1000f64);

        stdout
            .write_all(
                format!(
                    "{}Sent chunk {}/{} ({})",
                    vec![0; 50].iter().map(|_| " ").collect::<String>(),
                    chunk_index,
                    total_chunks,
                    human_bytes(upload_speed)
                )
                .as_bytes(),
            )
            .unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        // we increase before incase a SetChunkIndex packet arrives
        chunk_index += 1;

        // wait for next packet, could be next, UploadComplete or SetChunkIndex and the continue
        let msg = match socket.read() {
            Ok(msg) => msg,
            Err(e) => return tracing::error!("failed to read: {e:?}"),
        };
        if let Message::Binary(bin) = msg {
            let packet = Packet::from_bytes(&bin).unwrap();
            match packet {
                Packet::Next => (),
                Packet::Json(JsonData::UploadComplete(file)) => {
                    tracing::info!(
                        "Successfully uploaded {} (Id: {})",
                        local.file_name().unwrap().to_string_lossy(),
                        file.id
                    );

                    break;
                }
                Packet::Json(JsonData::SetChunkIndex(data)) => {
                    tracing::debug!(
                        "Got SetChunkIndex, going from {} to {}",
                        chunk_index,
                        data.chunk_index
                    );
                    chunk_index = data.chunk_index
                }
                _ => return tracing::error!("Unexpected JSON message"),
            }
        } else {
            return tracing::error!("Got invalid Message type");
        }

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
    }

    socket.close(None).unwrap();

    if let Some(access) = access {
        // we can borrow the access level from the access subcommand
        crate::access::access(app, FileIdentifier::Path(PathBuf::from(path)), access);
    }
}
