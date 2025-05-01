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
    pub async fn add_user(
        &self,
        uuid: &Uuid,
        email: &str,
        name: &str,
        subscribed_at: &DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4);"#,
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
