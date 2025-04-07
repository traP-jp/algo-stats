use anyhow::Result;
use async_trait::async_trait;
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct PersistRepositoryImpl {
    pool: MySqlPool,
}

impl PersistRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        PersistRepositoryImpl { pool }
    }
}

#[async_trait]
impl crate::domain::persist_repository::PersistRepository for PersistRepositoryImpl {
    async fn get_users(&self) -> Result<Vec<crate::domain::dto::User>> {
        let users = sqlx::query_as::<_, crate::domain::dto::User>(
            "SELECT * FROM users"
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch users: {}", e))?;
        Ok(users)
    }

    async fn get_user(&self, trap_account_name: &str) -> Result<Option<crate::domain::dto::User>> {
        let user = sqlx::query_as::<_, crate::domain::dto::User>(
            "SELECT * FROM users WHERE trap_account_name = ?"
        )
            .bind(trap_account_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch user: {}", e))?;
        Ok(user)
    }

    async fn set_users(&self, users: Vec<crate::domain::dto::User>) -> Result<()> {
        let mut query_builder = sqlx::QueryBuilder::<sqlx::MySql>::new(
            r#"
            INSERT INTO users (
                `trap_account_name`,
                `atcoder_account_name`,
                `atcoder_rating`,
                `heuristic_rating`,
                `is_algo_team`,
                `is_active`,
                `grade`
            )
            "#
        );
        query_builder.push_values(users, |mut b, user| {
            b
                .push_bind(user.trap_account_name)
                .push_bind(user.atcoder_account_name)
                .push_bind(user.atcoder_rating)
                .push_bind(user.heuristic_rating)
                .push_bind(user.is_algo_team)
                .push_bind(user.is_active)
                .push_bind(user.grade);
        });
        query_builder
            .push(
                r#"
                ON DUPLICATE KEY UPDATE
                    `atcoder_account_name` = VALUES(`atcoder_account_name`),
                    `atcoder_rating` = VALUES(`atcoder_rating`),
                    `heuristic_rating` = VALUES(`heuristic_rating`),
                    `is_algo_team` = VALUES(`is_algo_team`),
                    `is_active` = VALUES(`is_active`),
                    `grade` = VALUES(`grade`)
                "#
            );
        let query = query_builder.build();
        query
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute query: {}", e))?;
        Ok(())
    }
}
