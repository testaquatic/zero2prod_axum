use std::sync::Arc;

use axum::extract::State;
use reqwest::StatusCode;

use crate::{app_state::AppState, domain::extractor::TokenData, error::AppError};

#[tracing::instrument("Logout", fields(user_id = %token_data.user_id) skip_all, err(Debug))]
#[utoipa::path(
    description = "로그아웃을 한다",
    summary = "로그아웃",
    post,
    path = "/admin/logout",
    responses((
        status = http::StatusCode::CREATED, description = "로그아웃 성공"
    ))
)]
pub async fn logout(
    State(app_state): State<Arc<AppState>>,
    token_data: TokenData,
) -> Result<StatusCode, AppError> {
    app_state
        .auth_token_service
        .logout(&app_state, &token_data)
        .await?;

    Ok(StatusCode::ACCEPTED)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(logout))]
pub struct LogoutOpenApiDoc;
