use std::sync::Arc;

use anyhow::Context;
use axum::extract::FromRequestParts;
use jsonwebtoken::{DecodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};

use crate::{app_state::AppState, domain::credential::Claims, error::AppError};

/// Basic 인증 관련 추출자
pub struct TokenData {
    pub claims: Claims,
}

impl FromRequestParts<Arc<AppState>> for TokenData {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_credentials(parts).await?;

        let token_data = jsonwebtoken::decode::<Claims>(
            token.expose_secret().as_bytes(),
            &DecodingKey::from_ed_pem(
                app_state
                    .auth_token_service
                    .token_secret_public_key
                    .expose_secret(),
            )
            .context("Failed to make decoding key")
            .map_err(AppError::UnexpectedError)?,
            &Validation::new(jsonwebtoken::Algorithm::EdDSA),
        )
        .map(|claims| TokenData {
            claims: claims.claims,
        })
        .context("Invalid Token")
        .map_err(AppError::AuthError)?;

        app_state
            .auth_token_service
            .get_user_id_by_claims(app_state, &token_data.claims)
            .await?;

        let now = chrono::Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AppError::AuthError(anyhow::anyhow!("Expired Token")));
        }

        Ok(token_data)
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
