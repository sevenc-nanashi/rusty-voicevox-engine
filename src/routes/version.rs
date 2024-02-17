use axum::Json;

pub async fn get() -> Json<String> {
    Json(env!("CARGO_PKG_VERSION").to_string())
}
