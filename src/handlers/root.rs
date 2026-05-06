use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Msg {
    msg: &'static str,
}

pub async fn root() -> Json<Msg> {
    Json(Msg { msg: "ajisai" })
}
