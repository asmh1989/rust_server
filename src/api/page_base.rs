use async_trait::async_trait;
use serde_json::Value;

use serde::{Deserialize, Serialize};

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
    #[serde(rename = "s_version")]
    pub version: Option<String>,
    #[serde(rename = "s_project")]
    pub project: Option<u32>,
}

#[async_trait]
pub trait PageBase {
    async fn query(&self, info: &QueryInfo) -> Result<Value, String>;
    async fn update(&self, user: &str, params: &str) -> Result<(), String>;
    async fn delete(&self, user: &str, id: u32) -> Result<(), String>;
}

pub struct NotFoundPage;

#[async_trait]
impl PageBase for NotFoundPage {
    async fn query(&self, _info: &QueryInfo) -> Result<Value, String> {
        Err("not found".to_string())
    }

    async fn update(&self, _user: &str, _params: &str) -> Result<(), String> {
        Err("not found".to_string())
    }

    async fn delete(&self, _user: &str, _id: u32) -> Result<(), String> {
        Err("not found".to_string())
    }
}
