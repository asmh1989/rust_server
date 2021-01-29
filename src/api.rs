pub mod mdm45;
pub mod page_base;
pub mod project;

use crate::http_response::{response_error, response_ok, response_success};
use crate::mysql_find_one;
use actix_identity::Identity;
use actix_web::{delete, get, post, web, HttpResponse, Result};
use log::info;
use page_base::{NotFoundPage, PageBase};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use self::mdm45::Mdm45Page;
use self::project::ProjectPage;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListData<T> {
    #[serde(rename = "currPage")]
    pub current_page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    #[serde(rename = "pageTotal")]
    pub total: u64,
    #[serde(rename = "list")]
    pub page_list: Vec<T>,
}

#[derive(Deserialize, Debug)]
pub struct QueryInfo {
    pub limit: u32,
    pub page: u32,
}

pub async fn check_user(id: Identity) -> Result<String, String> {
    let user = id.identity();
    if user.is_none() {
        return Err("请先登录".to_string());
    }

    let username = user.unwrap();

    let result = mysql_find_one!(
        String,
        &format!("select name from sys_user where username = '{}'", username)
    )?;

    Ok(result)
}

#[inline]
async fn _query(mode: &str, limit: u32, page: u32) -> Result<Value, String> {
    match mode {
        "project" => ProjectPage::query(limit, page).await,
        "mdm45" => Mdm45Page::query(limit, page).await,
        _ => NotFoundPage::query(limit, page).await,
    }
}

pub async fn _update(mode: &str, user: &str, body: &str) -> Result<(), String> {
    match mode {
        "project" => ProjectPage::update(user, body).await,
        "mdm45" => Mdm45Page::update(user, body).await,
        _ => NotFoundPage::update(user, body).await,
    }
}

pub async fn _delete(mode: &str, user: &str, id: u32) -> Result<(), String> {
    match mode {
        "project" => ProjectPage::delete(user, id).await,
        "mdm45" => Mdm45Page::delete(user, id).await,
        _ => NotFoundPage::delete(user, id).await,
    }
}

#[get("/{page}/list")]
pub async fn query(
    id: Identity,
    page: web::Path<(String,)>,
    info: web::Query<QueryInfo>,
) -> HttpResponse {
    info!("query info {:?}!", info);

    match check_user(id).await {
        Ok(_) => match _query(&page.into_inner().0, info.limit, info.page).await {
            Ok(d) => response_ok(d),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}

#[post("/{page}/update")]
pub async fn update(id: Identity, page: web::Path<(String,)>, req_body: String) -> HttpResponse {
    match check_user(id).await {
        Ok(user) => match _update(&page.into_inner().0, &user, &req_body).await {
            Ok(_) => response_success("成功"),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}

#[delete("/{page}/delete/{id}")]
pub async fn delete(id: Identity, path: web::Path<(String, u32)>) -> HttpResponse {
    let p = path.into_inner();
    match check_user(id).await {
        Ok(user) => match _delete(&p.0, &user, p.1).await {
            Ok(_) => response_success("成功"),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}
