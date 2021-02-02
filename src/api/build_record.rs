use crate::{
    mysql::{count, sql_page_str},
    mysql_query, result_err,
};

use super::page_base::{ListData, PageBase};
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct BuildRecord {
    pub id: Option<i64>,
    pub project_id: i64,
    pub revision: String,
    pub project_name: String,
    pub project_no: String,
    pub app_name: Option<String>,
    pub svn_url: String,
    pub build_user: String,
    pub build_time: DateTime<Utc>,
    pub build_result: String,
    pub version_code: i64,
    pub version_name: String,
    pub build_uuid: String,
    pub config_detail_file: String,
    pub is_release: Option<i64>,
    pub release_file_arch: Option<String>,
    pub config_tag: String,
}

pub struct BuildRecordPage;

#[async_trait]
impl PageBase for BuildRecordPage {
    #[inline]
    async fn query(&self, info: &super::page_base::QueryInfo) -> Result<serde_json::Value, String> {
        let mut w = r#"config_tag is not null and build_result is not null"#.to_string();

        let limit = info.limit.or(Some(20)).unwrap();
        let page = info.page.or(Some(1)).unwrap();

        if info.project.is_some() {
            w = format!("{} and project_id={}", w, info.project.unwrap());
        }

        if info.version.is_some() {
            let c = info.version.clone().unwrap();
            w = format!(
                "{} and (version_code like concat('%{}%') or version_name like concat('%{}%') )",
                w, &c, &c
            );
        }
        let sql = sql_page_str(
            &format!(
                r#"
select id, project_id, project_no, project_name, svn_url, revision, app_name, build_result, build_user, build_status, build_time, build_uuid, version_code, version_name,
is_release, release_file_arch, config_detail_file, config_tag
from tb_version_build_record where  {}
                order by id desc"#,
                &w
            ),
            limit,
            page,
        )?;

        let mut data: Vec<BuildRecord> = Vec::new();

        let count = count(&format!(
            "SELECT COUNT(id) FROM tb_version_build_record 
            where {}",
            &w
        ))
        .await?;

        mysql_query!(BuildRecord, data, &sql)?;

        Ok(serde_json::to_value(ListData::<BuildRecord> {
            current_page: page,
            page_size: limit,
            total: count,
            page_list: data,
        })
        .map_err(result_err!())?)
    }

    async fn update(&self, _user: &str, _params: &str) -> Result<(), String> {
        Err("not found".to_string())
    }

    async fn delete(&self, _user: &str, _id: u32) -> Result<(), String> {
        Err("not found".to_string())
    }
}
