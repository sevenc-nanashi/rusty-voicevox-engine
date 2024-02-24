use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error(pub String);

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error(format!("{:?}", e))
    }
}

#[derive(Serialize)]
pub struct ErrorJson {
    pub error: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorJson { error: self.0 })).into_response()
    }
}
