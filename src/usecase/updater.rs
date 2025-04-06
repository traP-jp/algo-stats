use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use anyhow::Result;


pub struct Updater<
    DU: crate::domain::detail_updater::DetailedInfoUpdater,
    AU: crate::domain::ac_account_updater::TrapMemberAcAccountUpdater,
    TR: crate::domain::traq_repository::TraqRepository,
    PR: crate::domain::persist_repository::PersistRepository,
> {
    detail_updater: DU,
    account_updater: AU,
    traq_repository: TR,
    persist_repository: Arc<PR>,
}

impl <DU, AU, TR, PR> Updater<DU, AU, TR, PR>
where
    DU: crate::domain::detail_updater::DetailedInfoUpdater,
    AU: crate::domain::ac_account_updater::TrapMemberAcAccountUpdater,
    TR: crate::domain::traq_repository::TraqRepository,
    PR: crate::domain::persist_repository::PersistRepository,
{
    pub fn new(
        detail_updater: DU,
        account_updater: AU,
        traq_repository: TR,
        persist_repository: Arc<PR>,
    ) -> Self {
        Self {
            detail_updater,
            account_updater,
            traq_repository,
            persist_repository,
        }
    }

    pub async fn serve(self) -> Result<()> {
        let scheduler = tokio_cron_scheduler::JobScheduler::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create scheduler: {}", e))?;
        let updater = Arc::new(self);
        scheduler
            .add(
                tokio_cron_scheduler::Job::new_async("0 0 4 * * Mon", move |_, _| {
                    let updater = updater.clone();
                    let future = Box::pin(async move {
                        updater.update()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to update: {}", e))
                            .unwrap();
                    });
                    future
                })
                    .map_err(|e| anyhow::anyhow!("Failed to create job: {}", e))?
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add job: {}", e))?;
        scheduler
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start scheduler: {}", e))?;
        Ok(())
    }

    pub async fn update(&self) -> Result<()> {
        let trap_members = self.traq_repository
            .get_members()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get members: {}", e))?
            .into_iter()
            .map(|member| {
                (
                    member.trap_account_name.clone(),
                    member
                )
            })
            .collect::<HashMap<_, _>>();
        let trap_members_with_ac_account = self.account_updater
            .get()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get members with ac account: {}", e))?;
        let atcoder_usernames = trap_members_with_ac_account
            .iter()
            .filter_map(|member| {
                member.ac_account_name.clone()
            })
            .collect::<Vec<_>>();
        let detailed_infos = self.detail_updater
            .get(atcoder_usernames)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get detailed info: {}", e))?;
        let mut users = vec![];
        for member in trap_members_with_ac_account {
            let trap_member = trap_members
                .get(&member.trap_account_name);
            let detailed_info = member.ac_account_name
                .as_ref()
                .map(|username| {
                    detailed_infos.get(username)
                })
                .flatten();
            let user = crate::domain::dto::User {
                trap_account_name: member.trap_account_name,
                atcoder_account_name: member.ac_account_name,
                atcoder_rating: detailed_info
                    .map(|info| {
                        if info.algo_rating.is_empty() {
                            0
                        } else {
                            info.algo_rating[info.algo_rating.len() - 1].new_rating
                        }
                    }),
                heuristic_rating: detailed_info
                    .map(|info| {
                        if info.heur_rating.is_empty() {
                            0
                        } else {
                            info.heur_rating[info.heur_rating.len() - 1].new_rating
                        }
                    }),
                is_algo_team: trap_member
                    .map(|member| {
                        member.is_algo_team
                    }),
                is_active: trap_member
                    .map(|member| {
                        member.is_active
                    }),
                grade: trap_member
                    .map(|member| {
                        member.grade.clone()
                    })
                    .flatten(),
            };
            users.push(user);
        }
        self.persist_repository
            .set_users(users)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set users: {}", e))?;
        Ok(())
    }
}
