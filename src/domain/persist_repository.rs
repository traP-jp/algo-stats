use async_trait::async_trait;
use anyhow::Result;
use super::dto::*;

#[async_trait]
pub trait PersistRepository: Send + Sync + 'static {
    async fn get_users(&self) -> Result<Vec<User>>;
    async fn set_users(&self, users: Vec<User>) -> Result<()>;
}
