use chrono::Utc;
use sqlx::postgres::PgQueryResult;

pub struct PostgresDatabase {
    pool: sqlx::PgPool,
}

impl PostgresDatabase {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "Saving new subscriber details in the database", skip_all)]
    pub async fn insert_subscriber(
        &self,
        email: &str,
        name: &str,
    ) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4);",
            uuid::Uuid::new_v4(),
            email,
            name,
            Utc::now(),
        )
        .execute(&self.pool)
        .await
    }
}
