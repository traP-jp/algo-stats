use anyhow::Result;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use traq::apis::{user_api, group_api, configuration};
use uuid::Uuid;

static EACH_QUERY_WAIT_TIME_MS: u64 = 200;

pub struct TraqRepositoryImpl {
    conf: configuration::Configuration,
}

#[async_trait]
impl crate::domain::traq_repository::TraqRepository for TraqRepositoryImpl {
    async fn get_members(&self) -> Result<Vec<crate::domain::entity::TrapMember>> {
        tracing::info!("Starting to fetch from traq");
        let users = user_api::get_users(
            &self.conf,
            Some(true),
            None,
        )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get users: {}", e))?
            .into_iter()
            .filter(|user| {
                !user.bot
            })
            .collect::<Vec<_>>();
        tokio::time::sleep(std::time::Duration::from_millis(EACH_QUERY_WAIT_TIME_MS)).await;
        let all_groups = group_api::get_user_groups(&self.conf)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get user groups: {}", e))?;
        tokio::time::sleep(std::time::Duration::from_millis(EACH_QUERY_WAIT_TIME_MS)).await;
        let algo_team_group_id = all_groups
            .iter()
            .find(|group| {
                group.name == "algorithm"
            })
            .ok_or_else(|| anyhow::anyhow!("Algo team group not found"))?
            .id;
        let algo_team_members_id = self
            .get_ids_by_group(&algo_team_group_id.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get algo team members: {}", e))?
            .into_iter()
            .collect::<HashSet<_>>();
        let grade_groups = Self::find_all_grade_groups(all_groups);
        let mut user_id_to_grade_group_name = HashMap::new();
        for group in grade_groups {
            let members = self
                .get_ids_by_group(&group.id.to_string())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get group members: {}", e))?;
            for member in members {
                user_id_to_grade_group_name.insert(member, group.name.clone());
            }
        }
        let traq_members = users
            .into_iter()
            .map(|user| {
                crate::domain::entity::TrapMember {
                    trap_account_name: user.name,
                    is_active: user.state == traq::models::UserAccountState::Active,
                    is_algo_team: algo_team_members_id.contains(&user.id),
                    grade: user_id_to_grade_group_name
                        .get(&user.id)
                        .cloned(),
                }
            })
            .collect::<Vec<_>>();
        Ok(traq_members)
    }
}

impl TraqRepositoryImpl {
    pub fn new(conf: configuration::Configuration) -> Self {
        Self { conf }
    }

    async fn get_ids_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<Uuid>> {
        let members = group_api::get_user_group_members(
            &self.conf,
            group_id,
        )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get user group members: {}", e))?;
        let ids = members
            .into_iter()
            .map(|member| member.id)
            .collect::<Vec<_>>();
        tokio::time::sleep(std::time::Duration::from_millis(EACH_QUERY_WAIT_TIME_MS)).await;
        Ok(ids)
    }

    fn find_all_grade_groups(
        all_groups: Vec<traq::models::UserGroup>,
    ) -> Vec<traq::models::UserGroup> {
        let reg = regex::Regex::new(r"^[0-9]{2}[BMRD]$")
            .expect("Failed to compile regex");
        let grade_groups = all_groups
            .into_iter()
            .filter(|group| {
                reg.is_match(&group.name)
            })
            .collect::<Vec<_>>();
        grade_groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traq_repository::TraqRepository as _;
    use traq::apis::configuration::Configuration;

    #[tokio::test]
    async fn test_get_members() {
        let access_token = std::env::var("BOT_ACCESS_TOKEN")
            .expect("BOT_ACCESS_TOKEN not set");
        let configuration = Configuration {
            bearer_access_token: Some(access_token),
            ..Default::default()
        };
        let traq_repository = TraqRepositoryImpl::new(configuration);
        let result = traq_repository.get_members().await;
        match result {
            Ok(members) => {
                assert!(!members.is_empty());
                for member in members.iter() {
                    println!("Member: {}", member.trap_account_name);
                    if let Some(grade) = &member.grade {
                        println!("Grade: {}", grade);
                    }
                    println!("Is active: {}", member.is_active);
                    println!("Is algo team: {}", member.is_algo_team);
                }
            }
            Err(e) => {
                panic!("Failed to get members: {}", e);
            }
        }
    }
}