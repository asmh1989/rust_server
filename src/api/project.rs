use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    http_response::{response_error, response_ok},
    mysql::{count, sql_page_str},
    mysql_query, result_err,
};

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
    pub total: u64,
    #[serde(rename = "list")]
    pub page_list: Vec<Project>,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub limit: u32,
    pub page: u32,
}

pub struct ProjectApi {}

#[inline]
async fn _query(limit: u32, page: u32) -> Result<Value, String> {
    let sql = sql_page_str(
        r#"
    select project_id, no, name, status, create_time, create_user, update_time, update_user, version_svn_url
    from tb_project where is_delete is null  or  is_delete != 'Y' and name is not null
    order by project_id desc 
            "#,
        limit,
        page,
    )?;

    let mut data: Vec<Project> = Vec::new();

    let count = count(
        "SELECT COUNT(project_id) FROM tb_project 
        where is_delete is null  or  is_delete != 'Y' and name is not null",
    )
    .await?;

    mysql_query!(Project, data, &sql)?;

    Ok(serde_json::to_value(ProjectData {
        current_page: page,
        page_size: limit,
        total: count,
        page_list: data,
    })
    .map_err(result_err!())?)
}

impl ProjectApi {
    pub async fn query(id: Identity, info: web::Query<Info>) -> HttpResponse {
        info!("query info {:?}!", info);

        if id.identity().is_none() {
            response_error("请先登录")
        } else {
            match _query(info.limit, info.page).await {
                Ok(d) => response_ok(d),
                Err(err) => response_error(&err),
            }
        }
    }
}
