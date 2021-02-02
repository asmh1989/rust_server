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
pub struct MdmConfig {
    pub id: Option<i64>,
    pub config_key: String,
    pub config_name: Option<String>,
    pub config_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    pub create_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_user: Option<String>,
    pub create_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<DateTime<Utc>>,
    pub category: String,
    pub module: String,
    pub sort: i64,
}

pub struct Mdm45ConfigPage;

#[async_trait]
impl PageBase for Mdm45ConfigPage {
    #[inline]
    async fn query(&self, info: &QueryInfo) -> Result<Value, String> {
        let limit = info.limit.or(Some(2000)).unwrap();
        let page = info.page.or(Some(1)).unwrap();

        _query(limit, page).await
    }

    async fn update(&self, user: &str, params: &str) -> Result<(), String> {
        let v = serde_json::from_str::<MdmConfig>(params).map_err(result_err!())?;
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
select  id, config_key, config_name, config_type, category, remark, create_user, create_time, update_user, update_time, module, sort
from tb_version_config_mdm45 order by id desc
            "#,
        limit,
        page,
    )?;

    let mut data: Vec<MdmConfig> = Vec::new();

    let count = count("SELECT COUNT(id) FROM tb_version_config_mdm45").await?;

    mysql_query!(MdmConfig, data, &sql)?;

    Ok(serde_json::to_value(ListData::<MdmConfig> {
        current_page: page,
        page_size: limit,
        total: count,
        page_list: data,
    })
    .map_err(result_err!())?)
}

pub async fn _update(user: &str, params: &MdmConfig) -> Result<(), String> {
    let remark: String = match params.remark.clone() {
        Some(x) => format!("'{}'", x),
        None => "null".to_string(),
    };

    let name: String = match params.config_name.clone() {
        Some(x) => format!("'{}'", x),
        None => "null".to_string(),
    };

    let mut sql = format!(
        "insert into tb_version_config_mdm45 (create_time, config_key, config_name, category, create_user, remark, module, sort, config_type)  
values (NOW() ,'{}', {}, {}, '{}', {}, '{}', {}, '{}')",
        params.config_key, name, params.category, user, remark, params.module, params.sort, params.config_type
    );

    if params.id.is_some() {
        sql = format!(
            r#"UPDATE tb_version_config_mdm45
SET config_key = '{}', config_name = {}, category = '{}', update_user = '{}',  remark = {}, module = '{}', sort = {} , config_type ='{}', update_time = NOW()
where id={} "#,
            params.config_key,
            name,
            params.category,
            user,
            remark,
            params.module,
            params.sort,
            params.config_type,
            params.id.unwrap()
        );
    }

    execute(&sql).await?;

    Ok(())
}

pub async fn _delete(user: &str, id: u32) -> Result<(), String> {
    execute(&format!(
        "UPDATE tb_version_config_mdm45 SET is_delete = 'Y', update_user = '{}', update_time = NOW()  where id={} ",
        user, id
    ))
    .await?;

    Ok(())
}
