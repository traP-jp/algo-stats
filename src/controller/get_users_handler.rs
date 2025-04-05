use axum::{
    extract::Extension,
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use std::sync::Arc;
use crate::domain::persist_repository::PersistRepository as _;

pub async fn handler(
    Extension(p_repo): Extension<Arc<crate::infra::persist_repository::PersistRepositoryImpl>>,
) -> Result<impl IntoResponse, StatusCode> {
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
