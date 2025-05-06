use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
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
    /// 사용자를 Postgres에 추가한다.
    #[tracing::instrument(skip_all)]
    pub async fn add_user(
        &self,
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
    .execute(self.pg_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

        Ok(())
    }

    /// 사용자 id와 토큰을 `subscription_tokens` 테이블에 저장한다.
    #[tracing::instrument(skip_all)]
    pub async fn store_token(
        &self,
        subscriber_id: &Uuid,
        subscription_token: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2);"#,
            subscription_token,
            subscriber_id
        )
        .execute(self.pg_pool.as_ref())
        .await?;

        Ok(())
    }

    /// 사용자의 `subscription_tokens`테이블의 `status` 컬럼을 `confirmed`로 변경한다.
    #[tracing::instrument(skip_all)]
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
    #[tracing::instrument(skip_all)]
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
