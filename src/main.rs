#![allow(dead_code)]

use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    error::{InternalError, JsonPayloadError, QueryPayloadError},
    middleware::Logger,
    post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use api::project::ProjectApi;
use http_response::{response_error, response_ok, response_success};
use log::info;
use params::LoginParams;
use rand::Rng;
use serde_json::Value;

mod api;
mod config;
mod http_response;
mod mysql;
mod params;
mod sha;

#[macro_export]
macro_rules! result_err {
    () => {
        |err| {
            info!("err = {}", err);
            format!("{:?}", err)
        }
    };
}

#[post("/test/post")]
async fn hello(req_body: String) -> impl Responder {
    info!("test response data = {}", req_body);
    response_ok(Value::String("hello world".to_string()))
}

async fn index(id: Identity) -> String {
    format!(
        "Hello {}",
        id.identity().unwrap_or_else(|| "Anonymous".to_owned())
    )
}

async fn login(id: Identity, params: web::Json<LoginParams>) -> HttpResponse {
    let result = mysql::login(&params.username, &params.password).await;
    match result {
        Ok(_) => {
            id.remember(params.username.clone());
            response_success("登录成功")
        }
        Err(err) => response_error(&err),
    }
}

async fn logout(id: Identity) -> HttpResponse {
    id.forget();
    response_success("退出成功")
}

fn post_error(err: JsonPayloadError, _: &HttpRequest) -> Error {
    let res = format!("{}", err);
    InternalError::from_response(err, response_error(&res)).into()
}

fn query_error(err: QueryPayloadError, _: &HttpRequest) -> Error {
    let res = format!("{}", err);
    InternalError::from_response(err, response_error(&res)).into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    config::init_config();

    // 数据库初始化
    let _ = crate::mysql::init(crate::mysql::URL).await;

    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%U %s %D"))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&private_key)
                    .name("JSESSIONID")
                    .secure(false),
            ))
            .service(hello)
            .service(web::resource("/jpm").route(web::get().to(index)))
            .service(
                web::scope("/jpm")
                    .service(
                        web::resource("/login")
                            .data(web::JsonConfig::default().error_handler(post_error))
                            .route(web::post().to(login)),
                    )
                    .service(web::resource("/logout").route(web::post().to(logout)))
                    .service(
                        web::resource("/project/list")
                            .app_data(
                                // change query extractor configuration
                                web::QueryConfig::default().error_handler(query_error),
                            )
                            .to(ProjectApi::query),
                    ),
            )
    })
    .bind(format!("0.0.0.0:{}", 8080))?
    .run()
    .await
}
