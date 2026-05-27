use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    domain::credential::{TokenResponse, UsernamePassword},
    error::AppError,
};

#[tracing::instrument(skip_all,fields(username = login_input.username, user_id = tracing::field::Empty), err(Debug))]
#[utoipa::path(
  description = "로그인을 위한 인증을 받는다.",
  summary = "로그인",
  post,
  path = "/login",
  request_body(content = inline(UsernamePassword), content_type = "application/json"),
  responses(
    (status = http::StatusCode::OK, description = "OK", body = TokenResponse),
    (status = http::StatusCode::UNPROCESSABLE_ENTITY, description = "누락되거나 유효하지 않은 필드가 있음"),
    (status = http::StatusCode::INTERNAL_SERVER_ERROR, description = "서버 내부 오류"),
    (status = http::StatusCode::UNAUTHORIZED, description = "로그인 데이터 유효성 검증 실패"),
  )
)]
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(login_input): Json<UsernamePassword>,
) -> Result<Json<TokenResponse>, AppError> {
    let user_id = app_state
        .credential_service
        .validate_credentials(&app_state.pg_pool, &login_input)
        .await?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(Json(TokenResponse {
        token: "123".to_string(),
        token_type: "Bearer".to_string(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(login))]
pub struct PostLoginOpenApiDoc;
