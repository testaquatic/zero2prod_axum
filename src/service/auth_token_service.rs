use anyhow::Context;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use secrecy::{ExposeSecret, SecretSlice};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    database::postgres::tokens::{get_user_id_from_by_id, save_token},
    domain::credential::Claims,
    service::error::ServiceError,
};

pub struct AuthTokenService {
    pub token_secret_private_key: SecretSlice<u8>,
    pub token_secret_public_key: SecretSlice<u8>,
}

impl AuthTokenService {
    pub async fn generate_token(
        &self,
        app_state: &AppState,
        user_id: &Uuid,
    ) -> Result<String, ServiceError> {
        let now = Utc::now();
        let claims = Claims {
            auth_id: Uuid::new_v4(),
            exp: (now + chrono::Duration::hours(12)).timestamp(),
            iat: now.timestamp(),
        };

        let token = jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::EdDSA),
            &claims,
            &EncodingKey::from_ed_pem(
                app_state
                    .auth_token_service
                    .token_secret_private_key
                    .expose_secret(),
            )
            .context("Failed to generate encoding key")
            .map_err(ServiceError::UnexpectedError)?,
        )
        .context("unexpected error")
        .map_err(ServiceError::UnexpectedError)?;

        save_token(&app_state.pg_pool, &app_state.moka_cache, &claims, user_id).await?;

        Ok(token)
    }

    pub async fn get_user_id_by_claims(
        &self,
        app_state: &AppState,
        claims: &Claims,
    ) -> Result<Option<Uuid>, ServiceError> {
        let user_id =
            get_user_id_from_by_id(&app_state.pg_pool, &app_state.moka_cache, &claims.auth_id)
                .await?;

        Ok(user_id)
    }
}
