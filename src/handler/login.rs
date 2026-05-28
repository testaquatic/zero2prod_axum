use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::Utc;
use secrecy::SecretString;
use utoipa::{
    Modify,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    domain::{
        credential::Claims,
        extractor::TokenData,
        form_data::LoginFormData,
        response::{AppErrorMessage, TokenResponse},
    },
    error::AppError,
    service::error::ServiceError,
};

#[tracing::instrument(skip_all,fields(username = login_input.username, user_id = tracing::field::Empty), err(Debug))]
#[utoipa::path(
  description = "로그인을 한다.",
  summary = "로그인",
  post,
  path = "/login",
  request_body(content = inline(LoginFormData), content_type = "application/json"),
  responses(
    (status = http::StatusCode::OK, description = "로그인에 성공하면 인증 토큰을 반환한다", body = TokenResponse),
    (status = http::StatusCode::UNPROCESSABLE_ENTITY, body = AppErrorMessage, description = "누락되거나 유효하지 않은 필드가 있음"),
    (status = http::StatusCode::INTERNAL_SERVER_ERROR, body = AppErrorMessage, description = "서버 내부 오류"),
    (status = http::StatusCode::UNAUTHORIZED, body = AppErrorMessage, description = "로그인 데이터 유효성 검증 실패"),
  ),
  security(("basicAuth" = [])),
  tags = ["Account"]
)]
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(login_input): Json<LoginFormData>,
) -> Result<Json<TokenResponse>, AppError> {
    let token = process_token_generation(&app_state, &login_input).await?;

    Ok(Json(TokenResponse {
        token,
        token_type: "Bearer".to_string(),
    }))
}

/// 토큰 생성과 관련한 절차를 수행한다.
async fn process_token_generation(
    app_state: &AppState,
    login_input: &LoginFormData,
) -> Result<SecretString, ServiceError> {
    let user_id = app_state
        .credential_service
        .validate_credentials(app_state, login_input)
        .await?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let claims = Claims {
        auth_id: Uuid::new_v4(),
        exp: (Utc::now()
            + chrono::Duration::seconds(app_state.auth_token_service.token_expiration_seconds))
        .timestamp(),
        iat: Utc::now().timestamp(),
    };

    let token = app_state.auth_token_service.generate_token(&claims).await?;

    let token_data = TokenData { claims, user_id };

    app_state
        .auth_token_service
        .save_token_info(app_state, &token_data)
        .await?;

    Ok(token)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(login), modifiers(&PostLoginOpenApiDoc))]
pub struct PostLoginOpenApiDoc;

impl Modify for PostLoginOpenApiDoc {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components: &mut utoipa::openapi::Components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}
