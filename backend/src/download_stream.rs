use axum::body::Bytes;
use futures_core::Stream;
use http_body::{Body, Frame, SizeHint};
use pin_project_lite::pin_project;
use std::{
    convert::Infallible,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::spawn;

use crate::{AppState, db, file_system::FSStream};

pin_project! {
    pub struct DownloadStream {
        #[pin]
        stream: FSStream,
        state: Arc<AppState>,
        file_id: String,
        completed: bool
    }

    impl PinnedDrop for DownloadStream {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();
            if *this.completed {
                let (state, id) = (this.state.clone(), this.file_id.clone());

                spawn(async move {
                    let file = match db::file::get_via_id(&state.db, &id).await {
                        Ok(f) => f,
                        Err(err) => {
                            tracing::error!("{err:?}");
                            return;
                        }
                    };

                    match db::file::increment_download_count(&file, &state.db, Some(1)).await {
                        Err(err) => {
                            tracing::error!("{err:?}");
                            return;
                        },
                        Ok(_) => (),
                    };

                    tracing::debug!("successfully incremented download count after a full download: {}", id);
                });
            } else {
                tracing::error!("client cancelled download: {}", *this.file_id);
            }
        }
    }
}

impl Body for DownloadStream {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let mut this = self.project();

        match this.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                let bytes = Bytes::from(chunk);
                Poll::Ready(Some(Ok(Frame::data(bytes))))
            }
            Poll::Ready(Some(Err(_e))) => Poll::Ready(None),
            Poll::Ready(None) => {
                *this.completed = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.completed
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

impl DownloadStream {
    pub fn new(stream: FSStream, file_id: String, state: Arc<AppState>) -> Self {
        DownloadStream {
            stream,
            state,
            file_id,
            completed: false,
        }
    }
}
