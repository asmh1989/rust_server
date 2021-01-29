pub mod mdm45;
pub mod project;
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
}
