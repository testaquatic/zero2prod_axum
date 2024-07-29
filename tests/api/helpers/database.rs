use sqlx::{postgres::PgQueryResult, Database, Row};
use uuid::Uuid;
use zero2prod_axum::{
    database::{
        postgres::{DatabaseSettingsPgExt, PostgresPool},
        Z2PADBError, Z2PADB,
    },
    settings::{DatabaseSettings, DefaultDB},
};

pub trait DefaultDBPoolTestExt: Z2PADB {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, Z2PADBError>;

    async fn create_db(
        database_settings: &DatabaseSettings,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn migrate(&self) -> Result<(), Z2PADBError>;

    async fn add_test_user(&self) -> Result<PgQueryResult, Z2PADBError>;

    async fn test_user(&self) -> Result<(String, String), Z2PADBError>;
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
        sqlx::query::<DefaultDB>(&format!(
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

    async fn add_test_user(&self) -> Result<PgQueryResult, Z2PADBError> {
        sqlx::query::<DefaultDB>(
            "
            INSERT INTO users (user_id, username, password)
            VALUES ($1, $2, $3);
            ",
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .execute(self.as_ref())
        .await
        .map_err(Z2PADBError::SqlxError)
    }

    // 테스트 사용자의 이름과 비밀번호를 반환한다.
    async fn test_user(&self) -> Result<(String, String), Z2PADBError> {
        let row = sqlx::query("SELECT username, password FROM users LIMIT 1;")
            .fetch_one(self.as_ref())
            .await
            .map_err(Z2PADBError::SqlxError)?;

        let username: String = row.try_get("username").map_err(Z2PADBError::SqlxError)?;
        let password: String = row.try_get("password").map_err(Z2PADBError::SqlxError)?;
        Ok((username, password))
    }
}
