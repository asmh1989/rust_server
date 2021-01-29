use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait PageBase {
    async fn query(limit: u32, page: u32) -> Result<Value, String>;
    async fn update(user: &str, params: &str) -> Result<(), String>;
    async fn delete(user: &str, id: u32) -> Result<(), String>;
}

pub struct NotFoundPage;

#[async_trait]
impl PageBase for NotFoundPage {
    async fn query(_limit: u32, _page: u32) -> Result<Value, String> {
        Err("not found".to_string())
    }

    async fn update(_user: &str, _params: &str) -> Result<(), String> {
        Err("not found".to_string())
    }

    async fn delete(_user: &str, _id: u32) -> Result<(), String> {
        Err("not found".to_string())
    }
}
