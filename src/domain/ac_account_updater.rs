use anyhow::Result;

use async_trait::async_trait;

#[async_trait]
pub trait TrapMemberAcAccountUpdater: Send + Sync + 'static{
    async fn get(&self) -> Result<Vec<crate::domain::entity::TrapMemberWithAcAccount>>;
}
