use sqlx::{
    postgres::{PgConnectOptions, PgQueryResult, PgRow},
    Executor, PgPool, Postgres,
};

use crate::{database::basic::Zero2ProdAxumDatabase, settings::DatabaseSettings};

use super::query::pg_save_subscriber;

pub struct PostgresPool {
    pool: PgPool,
}

impl Zero2ProdAxumDatabase for PostgresPool {
    type Z2PADBPool = Self;
    type DB = Postgres;
    fn connect(database_settings: &crate::settings::DatabaseSettings) -> Result<Self, sqlx::Error> {
        let pg_connect_options = database_settings.pg_connect_options_with_db();
        let pool = PgPool::connect_lazy_with(pg_connect_options);
        Ok(PostgresPool { pool })
    }

    async fn fetch_one(&self, query: &str) -> Result<PgRow, sqlx::Error> {
        self.pool.fetch_one(query).await
    }

    async fn save_subscriber(&self, email: &str, name: &str) -> Result<PgQueryResult, sqlx::Error> {
        pg_save_subscriber(&self.pool, email, name).await
    }
}

pub trait DatabaseSettingsExt {
    fn pg_connect_options_without_db(&self) -> PgConnectOptions;
    fn pg_connect_options_with_db(&self) -> PgConnectOptions;
}

impl DatabaseSettingsExt for DatabaseSettings {
    fn pg_connect_options_without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .username(&self.username)
            .password(&self.password)
            .host(&self.host)
            .port(self.port)
    }
    fn pg_connect_options_with_db(&self) -> PgConnectOptions {
        self.pg_connect_options_without_db()
            .database(&self.database_name)
    }
}
