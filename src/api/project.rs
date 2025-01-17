use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::info;
use mysql::execute;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    mysql::{self, count, sql_page_str},
    mysql_query, result_err,
};

use super::page_base::{ListData, PageBase, QueryInfo};

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

pub struct ProjectPage;

#[async_trait]
impl PageBase for ProjectPage {
    #[inline]
    async fn query(&self, info: &QueryInfo) -> Result<Value, String> {
        _query(info).await
    }

    async fn update(&self, user: &str, params: &str) -> Result<(), String> {
        let v = serde_json::from_str::<Project>(params).map_err(result_err!())?;

        _update(user, &v).await
    }

    async fn delete(&self, user: &str, id: u32) -> Result<(), String> {
        _delete(user, id).await
    }
}

#[inline]
async fn _query(info: &QueryInfo) -> Result<Value, String> {
    let limit = info.limit.or(Some(20)).unwrap();
    let page = info.page.or(Some(1)).unwrap();

    let mut w = "is_delete is null and name is not null".to_string();

    if info.query.is_some() {
        w = format!(
            "{} and name like concat('%{}%') ",
            w,
            info.query.clone().unwrap()
        );
    }

    let sql = sql_page_str(
        &format!(
            r#"
    select project_id, no, name, status, create_time, create_user, update_time, update_user, version_svn_url
    from tb_project where {}
    order by project_id desc 
            "#,
            w
        ),
        limit,
        page,
    )?;

    let mut data: Vec<Project> = Vec::new();

    let count = count(&format!(
        "SELECT COUNT(project_id) FROM tb_project 
        where {}",
        w
    ))
    .await?;

    mysql_query!(Project, data, &sql)?;

    Ok(serde_json::to_value(ListData::<Project> {
        current_page: page,
        page_size: limit,
        total: count,
        page_list: data,
    })
    .map_err(result_err!())?)
}

pub async fn _update(user: &str, params: &Project) -> Result<(), String> {
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
SET no = '{}', name = '{}', status = {}, update_user = '{}',  version_svn_url = {}, update_time = NOW()
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
        "UPDATE tb_project SET is_delete = 'Y', update_user = '{}', update_time = NOW()  where project_id={} ",
        user, id
    ))
    .await?;

    Ok(())
}
