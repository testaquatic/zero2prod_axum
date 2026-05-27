use std::sync::Arc;

use anyhow::Context;
use axum::extract::FromRequestParts;
use base64::{Engine, engine::general_purpose::STANDARD};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    app_state::AppState,
    domain::credential::{Credentials, UsernamePassword},
    handler::error::AppError,
};

/// Basic 인증 관련 추출자
pub struct ExtractCredentials {
    pub username: String,
    pub password: SecretString,
    pub user_id: uuid::Uuid,
}

impl From<ExtractCredentials> for Credentials {
    fn from(value: ExtractCredentials) -> Self {
        Self {
            username: value.username,
            password: value.password,
            user_id: value.user_id,
        }
    }
}

impl FromRequestParts<Arc<AppState>> for ExtractCredentials {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // 일단 사용자명과 비밀번호를 추출한다
        let username_password = extract_credentials(parts).await?;

        // Clone 이전의 비밀번호 길이 검사
        // 256바이트라면 감당할만 하다
        // 설마 256자를 넘기는 비밀번호는 사용 안하겠지
        if username_password.password.expose_secret().len() > 256 {
            return Err(AppError::AuthError(anyhow::anyhow!("Password too long")));
        }

        // 사용자의 ID를 찾는다.
        let user_id = app_state
            .credential_service
            .validate_credentials(
                &app_state.pg_pool,
                &username_password.username,
                username_password.password.clone(),
            )
            .await?;
        tracing::Span::current().record("user_id", tracing::field::display(user_id));

        Ok(ExtractCredentials {
            username: username_password.username,
            password: username_password.password,
            user_id,
        })
    }
}

/// 요청에서 `Basic` 인증을 추출하는 함수
async fn extract_credentials(
    parts: &mut axum::http::request::Parts,
) -> Result<UsernamePassword, AppError> {
    let header_value = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .context("The 'Authorization' header was missing")
        .map_err(AppError::AuthError)?
        .to_str()
        .map_err(|e| AppError::AuthError(anyhow::anyhow!(e)))?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'")
        .map_err(AppError::AuthError)?;

    let decoded_bytes = STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials")
        .map_err(AppError::AuthError)?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not UTF-8")
        .map_err(AppError::AuthError)?;

    let mut credentials = decoded_credentials.splitn(2, ":");
    let user_name = credentials
        .next()
        .context("A username must be provided in `Basic` auth")
        .map_err(AppError::AuthError)?;

    let password = credentials
        .next()
        .context("A password must be provided in `Basic` auth")
        .map_err(AppError::AuthError)?;

    Ok(UsernamePassword {
        username: user_name.to_string(),
        password: SecretString::new(password.into()),
    })
}
