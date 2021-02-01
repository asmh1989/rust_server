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

pub struct Mdm45Page;

#[async_trait]
impl PageBase for Mdm45Page {
    #[inline]
    async fn query(&self, info: &QueryInfo) -> Result<Value, String> {
        _query(info.limit, info.page).await
    }

    async fn update(&self, user: &str, params: &str) -> Result<(), String> {
        let v = serde_json::from_str::<Version>(params).map_err(result_err!())?;
        _update(user, &v).await
    }

    async fn delete(&self, user: &str, id: u32) -> Result<(), String> {
        _delete(user, id).await
    }
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

pub async fn _update(user: &str, params: &Version) -> Result<(), String> {
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
