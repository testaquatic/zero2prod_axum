use std::borrow::Cow;

use secrecy::SecretString;
use sqlx::PgExecutor;

pub struct UserPasswordHash<'a> {
    pub user_id: uuid::Uuid,
    pub username: Cow<'a, str>,
    pub password_hash: SecretString,
}

#[tracing::instrument(name = "Get stored credentials", skip_all, err(Debug))]
pub async fn get_user_id_password_hash_from_username<'a>(
    pg_executor: impl PgExecutor<'_>,
    username: &'a str,
) -> Result<Option<UserPasswordHash<'a>>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pg_executor)
    .await
    .map(|row| {
        row.map(|user| UserPasswordHash {
            user_id: user.user_id,
            username: Cow::Borrowed(username),
            password_hash: SecretString::new(user.password_hash.into()),
        })
    })
}
