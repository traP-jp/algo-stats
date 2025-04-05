#![allow(non_snake_case, unused)]
use std::vec;

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

static TRAPORTFOLIO_WAIT_TIME_MS: u64 = 200;
static TRAPORTFOLIO_AC_ACCOUNT_TYPE_ID: i32 = 8;

#[derive(Debug, Clone, serde::Deserialize)]
struct TrapMemberMinimalDto {
    id: Uuid,
    name: String,
    realName: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TraportfolioAccountDto {
    id: Uuid,
    displayName: String,
    #[serde(rename = "type")]
    type_ : i32,
    url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TrapMemberDto {
    id: Uuid,
    name: String,
    realName: String,
    state: i32,
    bio: String,
    accounts: Vec<TraportfolioAccountDto>,
}

pub struct TrapMemberAcAccountUpdaterImpl {
    http_client: reqwest::Client,
}

impl TrapMemberAcAccountUpdaterImpl {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client");
        TrapMemberAcAccountUpdaterImpl { http_client }
    }
}

#[async_trait]
impl crate::domain::ac_account_updater::TrapMemberAcAccountUpdater for TrapMemberAcAccountUpdaterImpl {
    async fn get(&self) -> Result<Vec<crate::domain::entity::TrapMemberWithAcAccount>> {
        tracing::info!("Starting to fetch from traportfolio");
        // Fetch all members list
        let all_members_url = "https://portfolio.trap.jp/api/v1/users";
        let response = self
            .http_client
            .get(all_members_url)
            .header("Accept-Encoding", "gzip")
            .send()
            .await?
            .bytes()
            .await?;
        let mut gz = flate2::read::GzDecoder::new(&response[..]);
        let members: Vec<TrapMemberMinimalDto> = serde_json::from_reader(&mut gz)?;
        // Fetch detailed info for each member
        let mut results = vec![];
        for member in members {
            let url = format!("https://portfolio.trap.jp/api/v1/users/{}", member.id);
            let response = self
                .http_client
                .get(&url)
                .send()
                .await?;
            let text = response.text().await?;
            let member : TrapMemberDto = serde_json::from_str(&text)?;
            results.push(crate::domain::entity::TrapMemberWithAcAccount {
                trap_account_name: member.name,
                ac_account_name: member
                    .accounts
                    .iter()
                    .find(|account| account.type_ == TRAPORTFOLIO_AC_ACCOUNT_TYPE_ID)
                    .map(|account| account.displayName.clone()),
            });
            // Wait for a while to avoid overwhelming the server
            tokio::time::sleep(std::time::Duration::from_millis(TRAPORTFOLIO_WAIT_TIME_MS)).await;
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ac_account_updater::TrapMemberAcAccountUpdater as _;

    #[tokio::test]
    async fn test_get() {
        let updater = TrapMemberAcAccountUpdaterImpl {
            http_client: reqwest::Client::new(),
        };
        let result = updater.get().await;
        match result {
            Ok(data) => {
                let mut has_ac_account = 0;
                for member in data.iter() {
                    if let Some(ac_account_name) = &member.ac_account_name {
                        has_ac_account += 1;
                    }
                }
                println!("Members with AC account: {}", has_ac_account);
            }
            Err(e) => {
                assert!(false, "Failed to fetch data{}", e);
            }
        }
    }
}
