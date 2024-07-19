use sqlx::{
    postgres::{PgConnectOptions, PgQueryResult},
    PgPool, Postgres,
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
        let pg_connect_options = database_settings.connect_options_with_db();
        let pool = PgPool::connect_lazy_with(pg_connect_options);
        Ok(PostgresPool { pool })
    }

    #[tracing::instrument(
        name = "Saving new subscriber details in the database."
        skip_all,
    )]
    async fn insert_subscriber(
        &self,
        email: &str,
        name: &str,
    ) -> Result<PgQueryResult, sqlx::Error> {
        pg_save_subscriber(&self.pool, email, name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", &e);
                e
            })
    }
}

impl AsRef<PgPool> for PostgresPool {
    fn as_ref(&self) -> &PgPool {
        &self.pool
    }
}

impl PostgresPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub trait DatabaseSettingsExt {
    fn connect_options_without_db(&self) -> PgConnectOptions;
    fn connect_options_with_db(&self) -> PgConnectOptions;
}

impl DatabaseSettingsExt for DatabaseSettings {
    fn connect_options_without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .username(&self.username)
            .password(&self.password)
            .host(&self.host)
            .port(self.port)
    }
    fn connect_options_with_db(&self) -> PgConnectOptions {
        self.connect_options_without_db()
            .database(&self.database_name)
    }
}
