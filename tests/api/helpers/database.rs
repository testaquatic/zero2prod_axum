use sqlx::{postgres::PgQueryResult, Database, Executor};
use zero2prod_axum::{
    database::{
        postgres::{DatabaseSettingsPgExt, PostgresPool},
        Zero2ProdAxumDatabase,
    },
    settings::DatabaseSettings,
};

pub trait DefaultDBPoolTestExt: Zero2ProdAxumDatabase {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, sqlx::Error>;

    async fn create_db(
        &self,
        database_settings: &DatabaseSettings,
    ) -> Result<<Self::DB as Database>::QueryResult, sqlx::Error>;

    async fn fetch_one(&self, query: &str) -> Result<<Self::DB as Database>::Row, sqlx::Error>;

    async fn execute(
        &self,
        query: &str,
    ) -> Result<<Self::DB as Database>::QueryResult, sqlx::Error>;

    async fn migrate(&self) -> Result<(), sqlx::Error>;
}

impl DefaultDBPoolTestExt for PostgresPool {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, sqlx::Error> {
        let connect_options = database_settings.connect_options_without_db();
        let pool = sqlx::PgPool::connect_with(connect_options).await?;
        Ok(Self::new(pool))
    }

    async fn create_db(
        &self,
        database_settings: &DatabaseSettings,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let pool = Self::connect_without_db(database_settings).await?;

        pool.execute(format!(r#"CREATE DATABASE "{}""#, database_settings.database_name).as_str())
            .await
    }

    async fn fetch_one(
        &self,
        query: &str,
    ) -> Result<<<Self as Zero2ProdAxumDatabase>::DB as Database>::Row, sqlx::Error> {
        self.as_ref().fetch_one(query).await
    }

    async fn execute(&self, query: &str) -> Result<PgQueryResult, sqlx::Error> {
        self.as_ref().execute(query).await
    }

    async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(self.as_ref())
            .await
            .expect("Failed to migrate database.");
        Ok(())
    }
}
