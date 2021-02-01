pub mod mdm45;
pub mod page_base;
pub mod project;

use std::{collections::HashMap, sync::Arc};

use crate::http_response::{response_error, response_ok, response_success};
use crate::mysql_find_one;
use actix_identity::Identity;
use actix_web::{delete, get, post, web, HttpResponse, Result};
use log::info;
use once_cell::sync::OnceCell;
use page_base::{NotFoundPage, PageBase};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use self::mdm45::Mdm45Page;
use self::project::ProjectPage;

static PAGES: OnceCell<HashMap<String, Arc<dyn PageBase + Send + Sync>>> = OnceCell::new();
pub fn init() {
    let mut map: HashMap<String, Arc<dyn PageBase + Send + Sync>> = HashMap::new();
    map.insert("project".to_string(), Arc::new(ProjectPage));
    map.insert("mdm45".to_string(), Arc::new(Mdm45Page));
    let _ = PAGES.set(map);
}

pub fn get_page() -> &'static HashMap<String, Arc<dyn PageBase + Send + Sync>> {
    PAGES.get().unwrap()
}

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
async fn _query(mode: &str, limit: u32, index: u32) -> Result<Value, String> {
    let page = get_page();
    match page.get(mode).clone() {
        Some(p) => p.query(limit, index).await,
        None => NotFoundPage.query(limit, index).await,
    }
}

pub async fn _update(mode: &str, user: &str, body: &str) -> Result<(), String> {
    let page = get_page();
    match page.get(mode).clone() {
        Some(p) => p.update(user, body).await,
        None => NotFoundPage.update(user, body).await,
    }
}

pub async fn _delete(mode: &str, user: &str, id: u32) -> Result<(), String> {
    let page = get_page();
    match page.get(mode).clone() {
        Some(p) => p.delete(user, id).await,
        None => NotFoundPage.delete(user, id).await,
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
