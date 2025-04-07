mod domain;
mod infra;
mod usecase;
mod controller;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{Router, Extension};

use infra::{
    traq_repository::TraqRepositoryImpl,
    detail_updater::DetailUpdaterImpl,
    ac_account_updater::TrapMemberAcAccountUpdaterImpl,
};
use traq::apis::configuration::Configuration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let bot_access_token = std::env::var("TRAQ_BOT_ACCESS_TOKEN")
        .expect("TRAQ_BOT_ACCESS_TOKEN not set");
    let mysql_database = std::env::var("NS_MARIADB_DATABASE")
        .expect("NS_MARIADB_DATABASE not set");
    let mysql_database = urlencoding::encode(mysql_database.as_str());
    let mysql_user = std::env::var("NS_MARIADB_USER")
        .expect("NS_MARIADB_USER not set");
    let mysql_user = urlencoding::encode(mysql_user.as_str());
    let mysql_password = std::env::var("NS_MARIADB_PASSWORD")
        .expect("NS_MARIADB_PASSWORD not set");
    let mysql_password = urlencoding::encode(mysql_password.as_str());
    let mysql_host = std::env::var("NS_MARIADB_HOSTNAME")
        .expect("NS_MARIADB_HOSTNAME not set");
    let mysql_host = urlencoding::encode(mysql_host.as_str());
    let mysql_port = std::env::var("NS_MARIADB_PORT")
        .expect("NS_MARIADB_PORT not set");
    let mysql_port = urlencoding::encode(mysql_port.as_str());
    let mysql_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        mysql_user, mysql_password, mysql_host, mysql_port, mysql_database
    );
    let update_on_start = std::env::var("UPDATE_ON_START")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .expect("UPDATE_ON_START must be a boolean");
    let conf = Configuration {
        bearer_access_token: Some(bot_access_token),
        ..Default::default()
    };
    let traq_repository = TraqRepositoryImpl::new(conf);
    let detail_updater = DetailUpdaterImpl::new();
    let account_updater = TrapMemberAcAccountUpdaterImpl::new();
    let pool = sqlx::MySqlPool::connect(&mysql_url)
        .await
        .expect("Failed to connect to MySQL");
    let init_sql = include_str!("../init.sql");
    tracing::info!("Executing init.sql");
    sqlx::query(init_sql)
        .execute(&pool)
        .await
        .expect("Failed to execute init.sql");
    let persist_repository = infra::persist_repository::PersistRepositoryImpl::new(pool);
    let persist_repository = Arc::new(persist_repository);
    let updater = usecase::updater::Updater::new(
        detail_updater,
        account_updater,
        traq_repository,
        persist_repository.clone(),
    );
    if update_on_start {
        tracing::info!("Updating on start");
        updater.update().await.unwrap();
    }
    let app = Router::new()
        .route("/users", axum::routing::get(controller::get_users_handler::handler::<infra::persist_repository::PersistRepositoryImpl>))
        .route(
            "/rate/heuristic/{trap_account_name}",
            axum::routing::get(controller::get_rate_handler::heur_handler::<infra::persist_repository::PersistRepositoryImpl>),
        )
        .route(
            "/rate/algorithm/{trap_account_name}",
            axum::routing::get(controller::get_rate_handler::algo_handler::<infra::persist_repository::PersistRepositoryImpl>),
        )
        .layer(Extension(persist_repository));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.clone())
        .await
        .expect("Failed to start server");
    /*
    let either = futures::future::select(
        Box::pin(async move {
            updater.serve().await.unwrap();
        }),
        Box::pin(async move {
            axum::serve(listener, app).await.unwrap();
        }),
    ).await;
    match either {
        futures::future::Either::Left(_) => {
            println!("Updater finished");
        }
        futures::future::Either::Right(_) => {
            println!("Server finished");
        }
    }
*/
}
