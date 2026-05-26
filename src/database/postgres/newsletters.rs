use std::borrow::Cow;

use sqlx::PgExecutor;

pub struct ConfirmedSubscriber {
    pub email: String,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip_all, err(Debug))]
pub async fn get_confirmed_subscribers(
    pg_executor: impl PgExecutor<'_>,
) -> Result<Vec<ConfirmedSubscriber>, sqlx::Error> {
    sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pg_executor)
    .await
}

pub struct UserPasswordHash<'a> {
    pub user_id: uuid::Uuid,
    pub username: Cow<'a, str>,
    pub password_hash: Cow<'a, str>,
}

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
            password_hash: Cow::Owned(user.password_hash),
        })
    })
}
