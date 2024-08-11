use anyhow::Context;
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
};
use base64::Engine;
use secrecy::{ExposeSecret, Secret};
use tokio::task::spawn_blocking;

use crate::{
    database::{PostgresPool, UserCredential},
    utils::spawn_blocking_with_tracing,
};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

pub fn basic_authentication(headers: &http::HeaderMap) -> Result<Credentials, anyhow::Error> {
    // 헤더값이 존재한다면 유효한 UTF8 문자열이어야 한다.
    let header_value = headers
        .get(http::header::AUTHORIZATION)
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The `Authorization` header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    // `:` 구분자를 사용해서 두 개의 세그먼트로 나눈다.
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or(anyhow::anyhow!(
            "A username must be provide in 'Basic' auth."
        ))?
        .to_string();
    let password = credentials
        .next()
        .ok_or(anyhow::anyhow!(
            "A password must be provided in 'Basic' auth."
        ))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}

impl Credentials {
    // 발신자를 확인하고 발신자의 uuid를 반환한다.
    #[tracing::instrument(name = "Validate credentials", skip_all)]
    pub async fn validate_credentials(self, pool: &PostgresPool) -> Result<uuid::Uuid, AuthError> {
        let mut user_id = None;
        let mut expected_password_hash = Secret::new(
        // password   : S+D3Jr0Mfc4/u/M5Zi1zwCJCewL7caoEkkSRDSltM5g
        // salt       : J8FrwwtHYzHjgRZzf7wjog
        "$argon2id$v=19$m=19456,t=2,p=1$SjhGcnd3dEhZekhqZ1JaemY3d2pvZw$qRp9pYM490LRo6a4lCrqOSneFV2+x5rh2TVhlKMrueE".to_string()
    );

        if let Some(UserCredential {
            user_id: stored_user_id,
            password_hash: stored_password_hash,
        }) = pool
            .get_user_credentials(&self.username)
            .await
            .context("Failed to perform a query to retrieve stored credentials")
            .map_err(AuthError::UnexpectedError)?
        {
            user_id = Some(stored_user_id);
            expected_password_hash = stored_password_hash;
        }

        // 이 부분은 책의 접근 방향과 같이 함수로 작게 쪼개는 것이 맞다고 생각한다.
        // 단순히 다른 방향으로 구현해보고 싶었다.
        spawn_blocking(move || {
            // 그 뒤 스레드의 소유권을 클로저에 전달하고, 그 스코프 안에서 명시적으로 모든 계산을 실행한다.
            let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
                .context("Failed to parse hash in PHC string format.")
                .map_err(AuthError::UnexpectedError)?;

            tracing::info_span!("Verify password hash").in_scope(|| {
                // `https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id`를 보고 설정했다.
                Argon2::new(
                    argon2::Algorithm::Argon2id,
                    argon2::Version::V0x13,
                    Params::new(19456, 2, 1, None)
                        .context("Failed to get prams")
                        .map_err(AuthError::UnexpectedError)?,
                )
                .verify_password(
                    self.password.expose_secret().as_bytes(),
                    &expected_password_hash,
                )
                .context("Invalid password.")
                .map_err(AuthError::InvalidCredentials)
            })
        })
        .await
        // spawn_blocking은 실패할 수 있다.
        // 중첩된 Result를 갖는다.
        .context("Failed to spawn blocking tast.")
        .map_err(AuthError::UnexpectedError)??;

        // 저장소에서 크리덴셜을 찾으면 `Some`으로만 설정된다.
        // 따라서 기본 비밀번호가 제공된 비밀번호와 매칭하더라도 존재하지 않는 사용자는 인증하지 않는다.
        user_id
            .ok_or(anyhow::anyhow!("Unknown username."))
            .map_err(AuthError::InvalidCredentials)
    }
}

pub fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(19456, 2, 1, None).context("Failed to get prams")?,
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();

    Ok(Secret::new(password_hash))
}

#[tracing::instrument(name = "Change password", skip(password, pool))]
pub async fn change_password(
    user_id: uuid::Uuid,
    password: Secret<String>,
    pool: &PostgresPool,
) -> Result<(), anyhow::Error> {
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password.")?;
    pool.change_password(user_id, password_hash).await?;

    Ok(())
}
