use std::{error::Error, fmt::Display};

use axum::{http::StatusCode, response::Result};

#[derive(Debug)]
pub struct SimplyError {
    status_code: axum::http::StatusCode,
    reason: String,
    err: Option<Box<dyn Error + Send + Sync + 'static>>,
}

#[allow(unused)]
impl SimplyError {
    pub fn status<T>(status_code: StatusCode, reason: &str) -> Result<T, Self> {
        Err(Self {
            status_code,
            reason: reason.to_string(),
            err: None,
        })
    }

    pub fn full<T>(
        status_code: StatusCode,
        reason: &str,
        err: impl Error + Send + Sync + 'static,
    ) -> Result<T, Self> {
        Err(Self {
            status_code,
            reason: reason.to_string(),
            err: Some(Box::new(err)),
        })
    }

    pub fn construct<S: Into<String>>(
        status_code: StatusCode,
        reason: S,
        err: Option<Box<dyn Error + Send + Sync + 'static>>,
    ) -> Self {
        SimplyError {
            status_code,
            reason: reason.into(),
            err,
        }
    }
}

impl axum::response::IntoResponse for SimplyError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}: {:?}", self.reason, self.err);
        (self.status_code, self.reason).into_response()
    }
}

impl From<std::io::Error> for SimplyError {
    fn from(value: std::io::Error) -> Self {
        SimplyError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed IO operation".into(),
            err: Some(Box::new(value)),
        }
    }
}

impl From<sqlx::Error> for SimplyError {
    fn from(value: sqlx::Error) -> Self {
        SimplyError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed DB operation".into(),
            err: Some(Box::new(value)),
        }
    }
}

impl From<axum::http::Error> for SimplyError {
    fn from(value: axum::http::Error) -> Self {
        SimplyError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed HTTP operation".into(),
            err: Some(Box::new(value)),
        }
    }
}

impl From<qrcode::types::QrError> for SimplyError {
    fn from(value: qrcode::types::QrError) -> Self {
        SimplyError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed QRCode operation".into(),
            err: Some(Box::new(value)),
        }
    }
}

impl From<image::ImageError> for SimplyError {
    fn from(value: image::ImageError) -> Self {
        SimplyError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed Image operation".into(),
            err: Some(Box::new(value)),
        }
    }
}

impl From<axum::extract::multipart::MultipartError> for SimplyError {
    fn from(value: axum::extract::multipart::MultipartError) -> Self {
        SimplyError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Failed Multipart operation".into(),
            err: Some(Box::new(value)),
        }
    }
}

impl Display for SimplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.reason, self.status_code)
    }
}

impl Error for SimplyError {}

macro_rules! err {
    ($msg:expr) => {
        return Err(SimplyError::construct(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            $msg,
            None,
        ))
    };
    ($msg:expr, $code:ident) => {
        return Err(SimplyError::construct(
            axum::http::StatusCode::$code,
            $msg,
            None,
        ))
    };
    ($msg:expr, $code:ident, $err:expr) => {
        return Err(SimplyError::construct(
            axum::http::StatusCode::$code,
            $msg,
            Some(Box::new($err)),
        ))
    };
}
pub(crate) use err;
