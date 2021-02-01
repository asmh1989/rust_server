use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait PageBase {
    async fn query(&self, limit: u32, page: u32) -> Result<Value, String>;
    async fn update(&self, user: &str, params: &str) -> Result<(), String>;
    async fn delete(&self, user: &str, id: u32) -> Result<(), String>;
}

pub struct NotFoundPage;

#[async_trait]
impl PageBase for NotFoundPage {
    async fn query(&self, _limit: u32, _page: u32) -> Result<Value, String> {
        Err("not found".to_string())
    }

    async fn update(&self, _user: &str, _params: &str) -> Result<(), String> {
        Err("not found".to_string())
    }

    async fn delete(&self, _user: &str, _id: u32) -> Result<(), String> {
        Err("not found".to_string())
    }
}
