use axum::{
    extract::Extension,
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use std::sync::Arc;

pub async fn handler<PR>(
    Extension(p_repo): Extension<Arc<PR>>,
) -> Result<impl IntoResponse, StatusCode> 
where 
    PR: crate::domain::persist_repository::PersistRepository,
{
    tracing::info!("Received request to get users");
    let users = p_repo
        .get_users()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get users: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    tracing::info!("Successfully fetched users");
    Ok((StatusCode::OK, Json(users)))
}
