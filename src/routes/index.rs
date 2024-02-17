use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Index {
    version: String,
}

pub async fn get() -> Json<Index> {
    Json(Index {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
