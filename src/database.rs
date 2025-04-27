use std::{error::Error, sync::Arc};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// 데이터베이스의 작동을 추상화한다.
/// 트레이트나 인터페이스의 함수는 적어야 한다고 생각하지만 여기에서는 이 원칙을 포기한다.
pub trait ZDabaBase: Send + Sync + Clone {
    type Error: Send + Sync + Error;
    /// 구독자를 추가한다.
    fn add_user(
        &self,
        uuid: &Uuid,
        email: &str,
        name: &str,
        subscribed_at: &DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

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

impl ZDabaBase for ZPgPool {
    type Error = sqlx::Error;
    /// 사용자를 Postgres에 추가한다.
    async fn add_user(
        &self,
        uuid: &Uuid,
        email: &str,
        name: &str,
        subscribed_at: &DateTime<Utc>,
    ) -> Result<(), Self::Error> {
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
