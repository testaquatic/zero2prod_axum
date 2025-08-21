use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use crate::telemetry::spawn_blocking_with_tracing;

/// 인증과 관련된 오류를 표현하는 열거체
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// 사용자명과 비밀번호를 저장한다.
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

/// `Credentials`과 데이터베이스에 저장한 정보를 비교한다.
#[tracing::instrument(name = "Validate credentials", skip_all)]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &DatabaseConnection,
) -> Result<uuid::Uuid, AuthError> {
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html이 문서의 권장에 맞춰서 파라미터를 조정했다.
    let (user_id, expected_password_hash) = get_stored_credentials(pool, &credentials.username)
        .await?
        .map(|(stored_user_id, stored_password_hash)| (Some(stored_user_id), stored_password_hash))
        .unwrap_or_else(|| (
            None,
            SecretString::from(
                "$argon2id$v=19$m=19456,t=2,p=1$Z3NWOTh1SHk$n8OcMbBpmGuRmX2w5WnB1BbwHFfZhmJLxYyXzs1DYnY",
            ),
        ));

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")??;

    user_id.ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Unknown username.")))
}

/// DB에 users 테이블의 user_id와 password_hash를 질의한다.
#[tracing::instrument(name = "Get stored credentials", skip_all)]
async fn get_stored_credentials(
    pool: &DatabaseConnection,
    username: &str,
) -> Result<Option<(Uuid, SecretString)>, anyhow::Error> {
    let row = entities::users::Entity::find()
        .select_only()
        .columns([
            entities::users::Column::UserId,
            entities::users::Column::PasswordHash,
        ])
        .filter(Condition::all().add(entities::users::Column::Username.eq(username)))
        .into_tuple::<(Uuid, String)>()
        .one(pool)
        .await
        .context("Failed to perform a query to retrieve stored credentials.")?
        .map(|row| (row.0, SecretString::from(row.1)));

    Ok(row)
}

/// 비밀번호의 유효성을 확인한다.
#[tracing::instrument(name = "Verify password hash.", skip_all)]
fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}
