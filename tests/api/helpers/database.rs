use sqlx::{postgres::PgQueryResult, Database, Executor};
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
        &self,
        database_settings: &DatabaseSettings,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn fetch_one(&self, query: &str) -> Result<<Self::DB as Database>::Row, Z2PADBError>;

    async fn execute(
        &self,
        query: &str,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn migrate(&self) -> Result<(), Z2PADBError>;
}

impl DefaultDBPoolTestExt for PostgresPool {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, Z2PADBError> {
        let connect_options = database_settings.connect_options_without_db();
        let pool = sqlx::PgPool::connect_with(connect_options)
            .await
            .map_err(Z2PADBError::PoolError)?;
        Ok(Self::new(pool))
    }

    async fn create_db(
        &self,
        database_settings: &DatabaseSettings,
    ) -> Result<PgQueryResult, Z2PADBError> {
        let pool = Self::connect_without_db(database_settings).await?;

        pool.execute(format!(r#"CREATE DATABASE "{}""#, database_settings.database_name).as_str())
            .await
    }

    async fn fetch_one(
        &self,
        query: &str,
    ) -> Result<<<Self as Z2PADB>::DB as Database>::Row, Z2PADBError> {
        self.as_ref()
            .fetch_one(query)
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    async fn execute(&self, query: &str) -> Result<PgQueryResult, Z2PADBError> {
        self.as_ref()
            .execute(query)
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
}
