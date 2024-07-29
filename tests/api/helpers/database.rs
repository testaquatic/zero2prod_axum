use sqlx::{postgres::PgQueryResult, Database};
use uuid::Uuid;
use zero2prod_axum::{
    database::{
        postgres::{DatabaseSettingsPgExt, PostgresPool},
        Z2PADBError, Z2PADB,
    },
    settings::DatabaseSettings,
};

pub trait DefaultDBPoolTestExt: Z2PADB {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, Z2PADBError>;

    async fn create_db(
        database_settings: &DatabaseSettings,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn migrate(&self) -> Result<(), Z2PADBError>;

    async fn store_test_user(
        &self,
        uuid: &Uuid,
        username: &str,
        password_hash: &str,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;
}

impl DefaultDBPoolTestExt for PostgresPool {
    async fn connect_without_db(
        database_settings: &DatabaseSettings,
    ) -> Result<PostgresPool, Z2PADBError> {
        let connect_options = database_settings.connect_options_without_db();
        let pool = sqlx::PgPool::connect_with(connect_options)
            .await
            .map_err(Z2PADBError::PoolError)?;
        Ok(Self::new(pool))
    }

    async fn create_db(database_settings: &DatabaseSettings) -> Result<PgQueryResult, Z2PADBError> {
        let pool = Self::connect_without_db(database_settings).await?;
        sqlx::query(&format!(
            r#"CREATE DATABASE "{}""#,
            database_settings.database_name
        ))
        // .bind(&database_settings.database_name)
        .execute(pool.as_ref())
        .await
        .map_err(Z2PADBError::SqlxError)
    }

    async fn migrate(&self) -> Result<(), Z2PADBError> {
        sqlx::migrate!("./migrations")
            .run(self.as_ref())
            .await
            .expect("Failed to migrate database.");
        Ok(())
    }

    async fn store_test_user(
        &self,
        user_id: &Uuid,
        username: &str,
        password_hash: &str,
    ) -> Result<PgQueryResult, Z2PADBError> {
        sqlx::query!(
            r#"
            INSERT INTO users (user_id, username, password_hash)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            username,
            password_hash
        )
        .execute(self.as_ref())
        .await
        .map_err(Z2PADBError::SqlxError)
    }
}
