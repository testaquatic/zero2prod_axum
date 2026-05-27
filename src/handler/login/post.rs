use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa::{
    Modify,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

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

    let token = app_state
        .auth_token_service
        .generate_token(&app_state, &user_id)
        .await?;

    Ok(Json(TokenResponse {
        token: token.into(),
        token_type: "Bearer".to_string(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(login), modifiers(&PostLoginOpenApiDoc))]
pub struct PostLoginOpenApiDoc;

impl Modify for PostLoginOpenApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components: &mut utoipa::openapi::Components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}
