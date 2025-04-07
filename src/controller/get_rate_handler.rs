use axum::{
    extract::Extension,
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use std::sync::Arc;

pub async fn heur_handler<PR>(
    axum::extract::Path(trap_account_name): axum::extract::Path<String>,
    Extension(p_repo): Extension<Arc<PR>>,
) -> Result<impl IntoResponse, StatusCode> 
where
    PR: crate::domain::persist_repository::PersistRepository,
{
    tracing::info!("Received request for heuristic rate with account name: {}", trap_account_name);
    let user = p_repo
        .get_user(&trap_account_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let rate = user
        .map(|u| u.heuristic_rating)
        .flatten();
    tracing::info!("Returning heuristic rate for account name: {}", trap_account_name);
    Ok((StatusCode::OK, Json(rate)))
}

pub async fn algo_handler<PR>(
    axum::extract::Path(trap_account_name): axum::extract::Path<String>,
    Extension(p_repo): Extension<Arc<PR>>,
) -> Result<impl IntoResponse, StatusCode> 
where
    PR: crate::domain::persist_repository::PersistRepository,
{
    tracing::info!("Received request for algorithmic rate with account name: {}", trap_account_name);
    let user = p_repo
        .get_user(&trap_account_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let rate = user
        .map(|u| u.atcoder_rating)
        .flatten();
    tracing::info!("Returning algorithmic rate for account name: {}", trap_account_name);
    Ok((StatusCode::OK, Json(rate)))
}
