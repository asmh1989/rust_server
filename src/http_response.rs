use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Debug, Serialize, Deserialize)]
enum MyHttpReponse {
    #[serde(rename = "ok")]
    Ok(Value),
    #[serde(rename = "error")]
    Error(Value),
}

pub fn response_ok(value: Value) -> HttpResponse {
    HttpResponse::Ok().body(serde_json::to_string(&MyHttpReponse::Ok(value)).unwrap())
}

pub fn response_success(msg: &str) -> HttpResponse {
    HttpResponse::Ok()
        .body(serde_json::to_string(&MyHttpReponse::Ok(json!({ "msg": msg }))).unwrap())
}

pub fn response_error(msg: &str) -> HttpResponse {
    HttpResponse::Ok()
        .body(serde_json::to_string(&MyHttpReponse::Error(json!({ "msg": msg }))).unwrap())
}

pub fn response_error2(value: Value) -> HttpResponse {
    HttpResponse::Ok().body(serde_json::to_string(&MyHttpReponse::Error(json!(value))).unwrap())
}
