use actix_identity::Identity;
use actix_web::{delete, get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use log::info;
use mysql::execute;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    http_response::{response_error, response_ok, response_success},
    mysql::{self, count, sql_page_str},
    mysql_find_one, mysql_query, result_err,
};

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub project_id: Option<i64>,
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

pub async fn _update(user: &str, params: &web::Json<Project>) -> Result<(), String> {
    let version_svn_url: String = match params.version_svn_url.clone() {
        Some(x) => format!("'{}'", x),
        None => "null".to_string(),
    };

    let mut sql = format!(
        "insert into tb_project (no, name, status, create_user, version_svn_url)  
values ('{}', '{}', {}, '{}', {})",
        params.no, params.name, params.status, user, version_svn_url
    );

    if params.project_id.is_some() {
        sql = format!(
            r#"UPDATE tb_project 
SET no = '{}', name = '{}', status = {}, update_user = '{}',  version_svn_url = {}
where project_id={} "#,
            params.no,
            params.name,
            params.status,
            user,
            version_svn_url,
            params.project_id.unwrap()
        );
    }

    execute(&sql).await?;

    Ok(())
}

pub async fn _delete(user: &str, id: u32) -> Result<(), String> {
    execute(&format!(
        "UPDATE tb_project SET is_delete = 'Y', update_user = '{}'  where project_id={} ",
        user, id
    ))
    .await?;

    Ok(())
}

#[get("/project/list")]
pub async fn query(id: Identity, info: web::Query<Info>) -> HttpResponse {
    info!("query info {:?}!", info);

    match check_user(id).await {
        Ok(_) => match _query(info.limit, info.page).await {
            Ok(d) => response_ok(d),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}

#[post("/project/update")]
pub async fn update(id: Identity, params: web::Json<Project>) -> HttpResponse {
    info!("update project = {:?}", params);
    match check_user(id).await {
        Ok(user) => match _update(&user, &params).await {
            Ok(_) => response_success("成功"),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}

#[delete("/project/delete/{id}")]
pub async fn delete(id: Identity, path: web::Path<(u32,)>) -> HttpResponse {
    match check_user(id).await {
        Ok(user) => match _delete(&user, path.into_inner().0).await {
            Ok(_) => response_success("成功"),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}
