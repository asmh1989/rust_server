#![allow(dead_code)]

use log::info;

mod config;
mod mysql;
mod sha;

#[actix_web::main]
async fn main() {
    config::Config::get_instance();
    info!("Hello, world!");
}
