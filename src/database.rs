use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct ZPgPool {
    pub pg_pool: Arc<PgPool>,
}

impl ZPgPool {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }
}

impl ZPgPool {
    /// 사용자의 `subscription_tokens`테이블의 `status` 컬럼을 `confirmed`로 변경한다.
    pub async fn confirm_subscriber(&self, subscriber_id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1;"#,
            subscriber_id
        )
        .execute(self.pg_pool.as_ref())
        .await?;

        Ok(())
    }

    /// `subscription_token`을 입력하면 `subscriber_id`가 반환된다.
    pub async fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1;"#,
            subscription_token
        )
        .fetch_optional(self.pg_pool.as_ref())
        .await?;

        Ok(result.map(|r| r.subscriber_id))
    }
}

impl AsRef<PgPool> for ZPgPool {
    fn as_ref(&self) -> &PgPool {
        &self.pg_pool
    }
}

impl From<PgPool> for ZPgPool {
    fn from(pg_pool: PgPool) -> Self {
        Self::new(pg_pool)
    }
}

/// 사용자를 Postgres에 추가한다.
pub async fn insert_user_into_database(
    pg_executor: impl PgExecutor<'_>,
    uuid: &Uuid,
    email: &str,
    name: &str,
    subscribed_at: &DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, 'pending_confirmation');"#,
        uuid,
        email,
        name,
        subscribed_at
    )
    .execute(pg_executor)
    .await?;

    Ok(())
}

pub async fn select_uuid_pending_confirmation_email(
    pg_executor: impl PgExecutor<'_>,
    email: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT id FROM subscriptions WHERE email = $1 AND status = 'pending_confirmation';"#,
        email
    )
    .fetch_optional(pg_executor)
    .await?
    .map(|record| record.id);

    Ok(result)
}

pub async fn update_token(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscription_tokens SET subscription_token = $1 WHERE subscriber_id = $2;"#,
        subscription_token,
        subscriber_id
    )
    .execute(pg_executor)
    .await?;

    Ok(())
}

/// 사용자 id와 토큰을 `subscription_tokens` 테이블에 저장한다.
pub async fn store_token_in_database(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2);"#,
        subscription_token,
        subscriber_id
    )
    .execute(pg_executor)
    .await?;

    Ok(())
}

/// 사용자 id에 이미 토큰이 저장되어 있는지 확인한다.
pub async fn is_subscriber_already_has_token(
    pg_executor: impl PgExecutor<'_>,
    subscriber_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscription_token FROM subscription_tokens WHERE subscriber_id = $1;"#,
        subscriber_id
    )
    .fetch_optional(pg_executor)
    .await?;

    Ok(result.is_some())
}
