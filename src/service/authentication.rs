use anyhow::Context;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core},
};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    app_state::AppState,
    database::postgres::users::{
        get_user_id_password_hash_from_username, get_user_info_by_user_id, update_password_hash,
    },
    domain::{
        extractor::TokenData,
        form_data::{ChangePasswordFormData, LoginFormData},
    },
    service::error::ServiceError,
};

pub struct CredentialService;

impl CredentialService {
    #[tracing::instrument(name = "Validate credentials", skip_all, err(Debug))]
    pub async fn validate_credentials(
        &self,
        app_state: &AppState,
        username_password: &LoginFormData,
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

    pub async fn change_password(
        &self,
        app_state: &AppState,
        token_data: &TokenData,
        change_password_form_data: &ChangePasswordFormData,
    ) -> Result<(), ServiceError> {
        // 일단 길이부터 확인한다.
        if change_password_form_data.new_password.expose_secret().len() <= 12 {
            return Err(ServiceError::ValidationError(
                "Password too short".to_string(),
            ));
        }

        if change_password_form_data.new_password.expose_secret().len() >= 128 {
            return Err(ServiceError::ValidationError(
                "Password too long".to_string(),
            ));
        }

        // 비밀번호 두개는 같아야 한다.
        if change_password_form_data.new_password.expose_secret()
            != change_password_form_data.new_password_check.expose_secret()
        {
            return Err(ServiceError::ValidationError(
                "You entered two different new passwords".to_string(),
            ));
        }

        // user_id를 불러오고 기존의 비밀번호가 맞는지 확인한다.
        let user_info = get_user_info_by_user_id(&app_state.pg_pool, &token_data.user_id)
            .await?
            .context("No user data!")
            .map_err(ServiceError::UnexpectedError)?;

        let login_form_data = LoginFormData {
            username: user_info.username,
            password: change_password_form_data.current_password.clone(),
        };

        self.validate_credentials(app_state, &login_form_data)
            .await?;

        let new_password = change_password_form_data.new_password.clone();
        // 해시를 생성한다.
        let password_hash =
            spawn_blocking_with_tracing(move || compute_password_hash(new_password))
                .await
                .context("tokio join error")
                .map_err(ServiceError::UnexpectedError)??;

        update_password_hash(&app_state.pg_pool, &token_data.user_id, &password_hash).await?;

        Ok(())
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

fn compute_password_hash(password: SecretString) -> Result<SecretString, ServiceError> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    Argon2::default()
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .context("Failed to hash password")
        .map(|hash| hash.to_string().into())
        .map_err(ServiceError::UnexpectedError)
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
