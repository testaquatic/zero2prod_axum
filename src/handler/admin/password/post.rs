use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    domain::{extractor::TokenData, form_data::ChangePasswordFormData, response::AppErrorMessage},
    error::AppError,
};

#[tracing::instrument("Change password", skip_all, err(Debug))]
#[utoipa::path(
  description = "비밀번호 변경한다",
  summary = "비밀번호 변경",
  post,
  path = "/admin/password",
  request_body(content = ChangePasswordFormData),
  responses(
    (status = http::StatusCode::OK, description = "OK"),
    (status = http::StatusCode::UNAUTHORIZED, body = AppErrorMessage, description = "인증 오류: 접근 권한이 없음"),
    (status = http::StatusCode::UNPROCESSABLE_ENTITY, body = AppErrorMessage, description = "누락되거나 유효하지 않은 필드가 있음"),
    (status = http::StatusCode::INTERNAL_SERVER_ERROR, body = AppErrorMessage, description = "서버 내부 오류"),
  )
)]
pub async fn change_password(
    State(app_state): State<Arc<AppState>>,
    token_data: TokenData,
    Json(change_password_form_data): Json<ChangePasswordFormData>,
) -> Result<(), AppError> {
    app_state
        .credential_service
        .change_password(&app_state, &token_data, &change_password_form_data)
        .await?;
    Ok(())
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(change_password))]
pub struct ChangePasswordOpenApiDoc;
