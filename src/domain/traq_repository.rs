use anyhow::Result;
use async_trait::async_trait;
use super::entity::*;

#[async_trait]
pub trait TraqRepository: Send + Sync + 'static {
    async fn get_members(&self) -> Result<Vec<TrapMember>>;
}
