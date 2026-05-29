use std::sync::Arc;

use axum::{Json, extract::State};
use reqwest::StatusCode;

use crate::{
    app_state::AppState,
    domain::{
        extractor::TokenData,
        response::{AppErrorMessageResponse, UserInfoResponse},
    },
    error::AppError,
};

#[tracing::instrument(name = "Get admin dashboard", skip_all, err(Debug))]
#[utoipa::path(
    get,
    path = "/admin/dashboard",
    tag = "Admin",
    summary = "관리자 화면에서 필요한 정보",
    description = "관리자 화면에서 필요한 정보를 전송한다.",
    params(
        ("Authorization" = String, Header, description = "Bearer authentication", example = "Bearer <token>"),
    ),
    responses(
        (status = StatusCode::OK, description = "Dashboard info", body = UserInfoResponse),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "접근 권한이 없음",
            body = AppErrorMessageResponse,
            example = json!({
                "status": StatusCode::UNAUTHORIZED.to_string(),
                "message": "인증 오류: 사용자 정보를 찾을 수 없습니다",
            })
        ),
    ),
    security(("bearerAuth" = [])),
    tags = ["Admin"]
)]
pub async fn get_admin_dashboard(
    State(app_state): State<Arc<AppState>>,
    token_data: TokenData,
) -> Result<Json<UserInfoResponse>, AppError> {
    let user_info = app_state
        .dashboard_service
        .get_admin_dashboard(&app_state, &token_data)
        .await?;

    Ok(Json(user_info))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_admin_dashboard))]
pub struct GetAdminDashboardOpenApiDoc;
