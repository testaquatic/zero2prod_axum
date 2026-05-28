use std::sync::Arc;

use anyhow::Context;
use axum::extract::FromRequestParts;
use jsonwebtoken::{DecodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use crate::{
    app_state::AppState, domain::credential::Claims, error::AppError, service::error::ServiceError,
};

/// Bearer 인증 관련 추출자
pub struct TokenData {
    pub user_id: Uuid,
    pub claims: Claims,
}

impl FromRequestParts<Arc<AppState>> for TokenData {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_credentials(parts).await?;
        let tokendata = validate_token(app_state, token).await?;
        Ok(tokendata)
    }
}

/// 요청에서 Bearer 인증 데이터를 추출한다.
async fn extract_credentials(
    parts: &mut axum::http::request::Parts,
) -> Result<SecretString, AppError> {
    let token = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .context("The 'Authorization' header was missing")
        .map_err(AppError::AuthError)?
        .to_str()
        .map_err(|e| AppError::AuthError(anyhow::anyhow!(e)))?;

    let token = token
        .strip_prefix("Bearer ")
        .context("No token")
        .map_err(AppError::AuthError)?;

    Ok(SecretString::new(token.to_string().into()))
}

/// 토큰이 유효한지 확인하고 `TokenData`를 반환하던다
pub async fn validate_token(
    app_state: &AppState,
    token: SecretString,
) -> Result<TokenData, ServiceError> {
    let decoding_key = DecodingKey::from_ed_pem(
        app_state
            .auth_token_service
            .token_secret_public_key
            .expose_secret(),
    )
    .context("Failed to make decoding key")
    .map_err(ServiceError::UnexpectedError)?;

    let claims = tokio::task::spawn_blocking(move || {
        jsonwebtoken::decode::<Claims>(
            token.expose_secret().as_bytes(),
            &decoding_key,
            &Validation::new(jsonwebtoken::Algorithm::EdDSA),
        )
        .map(|claims| claims.claims)
        .context("Invalid Token")
        .map_err(ServiceError::AuthError)
    })
    .await
    .context("tokio join error")
    .map_err(ServiceError::UnexpectedError)??;

    let user_id = app_state
        .auth_token_service
        .get_user_id_by_claims(app_state, &claims)
        .await?
        .context("No user id")
        .map_err(ServiceError::AuthError)?;

    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(ServiceError::AuthError(anyhow::anyhow!("Expired Token")));
    }

    Ok(TokenData { user_id, claims })
}
