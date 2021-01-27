use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};

use crate::http_response::{response_error, response_success};

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub project_id: i64,
    pub no: String,
    pub name: String,
    pub create_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_user: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    pub status: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_svn_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectData {
    #[serde(rename = "currPage")]
    pub current_page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    #[serde(rename = "pageTotal")]
    pub total: u32,
    #[serde(rename = "list")]
    pub page_list: Vec<Project>,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub limit: u32,
    pub page: u32,
}

pub struct ProjectApi {}

impl ProjectApi {
    pub async fn query(id: Identity, info: web::Query<Info>) -> HttpResponse {
        info!("query info {:?}!", info);

        if id.identity().is_none() {
            response_error("请先登录")
        } else {
            response_success("success")
        }
    }
}
