use anyhow::Context;
use jsonwebtoken::{EncodingKey, Header};
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    database::postgres::tokens::{delete_token, get_user_id_by_token_id, save_token_info},
    domain::{credential::Claims, extractor::TokenData},
    service::error::ServiceError,
};

pub struct AuthTokenService {
    pub token_secret_private_key: SecretSlice<u8>,
    pub token_secret_public_key: SecretSlice<u8>,
}

impl AuthTokenService {
    /// JWT 토큰을 생성한다.
    pub async fn generate_token(&self, claims: &Claims) -> Result<SecretString, ServiceError> {
        let encoding_key = EncodingKey::from_ed_pem(&self.token_secret_private_key.expose_secret())
            .context("Failed to generate encoding key")
            .map_err(ServiceError::UnexpectedError)?;

        let token_claims = claims.clone();
        let token = tokio::task::spawn_blocking(move || {
            jsonwebtoken::encode(
                &Header::new(jsonwebtoken::Algorithm::EdDSA),
                &token_claims,
                &encoding_key,
            )
            .context("unexpected error")
            .map_err(ServiceError::UnexpectedError)
        })
        .await
        .context("tokio join error")
        .map_err(ServiceError::UnexpectedError)??;

        Ok(token.into())
    }

    /// 토큰 정보를 저장한다.
    /// 토큰 자체는 저장하지 않는다.
    pub async fn save_token_info(
        &self,
        app_state: &AppState,
        token_data: &TokenData,
    ) -> Result<(), ServiceError> {
        save_token_info(&app_state.pg_pool, &app_state.moka_cache, &token_data).await?;
        Ok(())
    }

    /// `Claims`로부터 user_id를 불러온다.
    pub async fn get_user_id_by_claims(
        &self,
        app_state: &AppState,
        claims: &Claims,
    ) -> Result<Option<Uuid>, ServiceError> {
        let user_id =
            get_user_id_by_token_id(&app_state.pg_pool, &app_state.moka_cache, &claims.auth_id)
                .await?;

        Ok(user_id)
    }

    pub async fn logout(
        &self,
        app_state: &AppState,
        token_data: &TokenData,
    ) -> Result<(), ServiceError> {
        delete_token(
            &app_state.pg_pool,
            &app_state.moka_cache,
            &token_data.claims.auth_id,
        )
        .await?;
        Ok(())
    }
}
