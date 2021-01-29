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

use super::{ListData, QueryInfo};

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub id: Option<i64>,
    pub revision: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    pub create_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_user: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    pub version_prop: i32,
}

#[inline]
async fn _query(limit: u32, page: u32) -> Result<Value, String> {
    let sql = sql_page_str(
        r#"
select id, revision, name, version_prop, create_user, create_time,  update_time, update_user, remark, is_delete
from tb_version_mdm45 where is_delete is null  and name is not null and version_prop is not null
order by id desc
            "#,
        limit,
        page,
    )?;

    let mut data: Vec<Version> = Vec::new();

    let count = count(
        "SELECT COUNT(id) FROM tb_version_mdm45 
        where is_delete is null  or  is_delete != 'Y' and name is not null",
    )
    .await?;

    mysql_query!(Version, data, &sql)?;

    Ok(serde_json::to_value(ListData::<Version> {
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

pub async fn _update(user: &str, params: &web::Json<Version>) -> Result<(), String> {
    let remark: String = match params.remark.clone() {
        Some(x) => format!("'{}'", x),
        None => "null".to_string(),
    };

    let mut sql = format!(
        "insert into tb_version_mdm45 (create_time, revision, name, version_prop, create_user, remark)  
values (NOW() ,'{}', '{}', {}, '{}', {})",
        params.revision, params.name, params.version_prop, user, remark
    );

    if params.id.is_some() {
        sql = format!(
            r#"UPDATE tb_version_mdm45 
SET revision = '{}', name = '{}', version_prop = {}, update_user = '{}',  remark = {}, update_time = NOW()
where id={} "#,
            params.revision,
            params.name,
            params.version_prop,
            user,
            remark,
            params.id.unwrap()
        );
    }

    execute(&sql).await?;

    Ok(())
}

pub async fn _delete(user: &str, id: u32) -> Result<(), String> {
    execute(&format!(
        "UPDATE tb_version_mdm45 SET is_delete = 'Y', update_user = '{}', update_time = NOW()  where id={} ",
        user, id
    ))
    .await?;

    Ok(())
}

#[get("/mdm45/list")]
pub async fn query(id: Identity, info: web::Query<QueryInfo>) -> HttpResponse {
    info!("query info {:?}!", info);

    match check_user(id).await {
        Ok(_) => match _query(info.limit, info.page).await {
            Ok(d) => response_ok(d),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}

#[post("/mdm45/update")]
pub async fn update(id: Identity, params: web::Json<Version>) -> HttpResponse {
    info!("update mdm45 = {:?}", params);
    match check_user(id).await {
        Ok(user) => match _update(&user, &params).await {
            Ok(_) => response_success("成功"),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}

#[delete("/mdm45/delete/{id}")]
pub async fn delete(id: Identity, path: web::Path<(u32,)>) -> HttpResponse {
    match check_user(id).await {
        Ok(user) => match _delete(&user, path.into_inner().0).await {
            Ok(_) => response_success("成功"),
            Err(err) => response_error(&err),
        },
        Err(err) => response_error(&err),
    }
}
