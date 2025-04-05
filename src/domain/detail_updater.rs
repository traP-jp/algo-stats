use anyhow::Result;
use async_trait::async_trait;
use super::entity::*;
use std::collections::HashMap;

#[async_trait]
pub trait DetailedInfoUpdater: Send + Sync + 'static {
    async fn get(&self, usernames: Vec<String>) -> Result<HashMap<String, AcDetailedInfo>>;
}
