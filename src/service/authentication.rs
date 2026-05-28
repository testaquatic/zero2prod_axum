use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    app_state::AppState, database::postgres::users::get_user_id_password_hash_from_username,
    domain::form_data::UsernamePasswordFormData, service::error::ServiceError,
};

pub struct CredentialService;

impl CredentialService {
    #[tracing::instrument(name = "Validate credentials", skip_all, err(Debug))]
    pub async fn validate_credentials(
        &self,
        app_state: &AppState,
        username_password: &UsernamePasswordFormData,
    ) -> Result<uuid::Uuid, ServiceError> {
        let user_password_hash = get_user_id_password_hash_from_username(
            &app_state.pg_pool,
            &username_password.username,
        )
        .await?;
        let user_id = user_password_hash.as_ref().map(|user| user.user_id);
        let user_password_hash = user_password_hash
            .map(|user| user.password_hash)
            // 소요시간 분석 공격을 회피하기 위해서, 일단 비밀번호 관련 연산을 수행하도록 한다.
            .unwrap_or(
                SecretString::new("$argon2id$v=19$m=19456,t=2,p=1$Ty9NdHNhNzQ$dBqxxXkpnU8ob6RgDsVlPw7BsC76W0/v0z7JpdEkJds".to_string().into())
            );

        let password = username_password.password.clone();
        spawn_blocking_with_tracing(move || verify_password_hash(password, user_password_hash))
            .await
            .context("Failed to spawn blocking task")
            .map_err(ServiceError::UnexpectedError)??;

        user_id
            .with_context(|| format!("Unknown username: {}", username_password.username))
            .map_err(ServiceError::AuthError)
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all, err(Debug))]
fn verify_password_hash(
    password: SecretString,
    password_hash: SecretString,
) -> Result<(), ServiceError> {
    let password_hash = PasswordHash::new(password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format")
        .map_err(ServiceError::UnexpectedError)?;
    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &password_hash)
        .context("Invalid password")
        .map_err(ServiceError::AuthError)
}

fn spawn_blocking_with_tracing<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
