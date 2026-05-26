use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgExecutor;

use crate::{
    database::postgres::newsletters::get_user_id_password_hash_from_username,
    service::error::ServiceError,
};

pub struct CredentialService;

impl CredentialService {
    pub async fn validate_credentials(
        &self,
        pg_executor: impl PgExecutor<'_>,
        username: &str,
        password: &SecretString,
    ) -> Result<uuid::Uuid, ServiceError> {
        let user = get_user_id_password_hash_from_username(pg_executor, username)
            .await
            .map_err(|e| ServiceError::UnexpectedError(e.into()))?
            .ok_or_else(|| {
                ServiceError::AuthError(anyhow::anyhow!("Unknown username: {}", username))
            })?;

        let password_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| ServiceError::UnexpectedError(anyhow::anyhow!(e)))?;
        Argon2::default()
            .verify_password(password.expose_secret().as_bytes(), &password_hash)
            .map_err(|e| ServiceError::AuthError(anyhow::anyhow!(e)))?;

        Ok(user.user_id)
    }
}
