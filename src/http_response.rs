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
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&MyHttpReponse::Ok(value)).unwrap())
}

pub fn response_error(msg: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&MyHttpReponse::Error(json!({ "msg": msg }))).unwrap())
}
