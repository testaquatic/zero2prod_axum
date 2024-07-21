use secrecy::ExposeSecret;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgQueryResult, PgSslMode},
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
        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(pg_connect_options);
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
        let ssl_mod = if self.require_ssl {
            PgSslMode::Require
        } else {
            // 암호화된 커넥션을 시도한다.
            // 실패하면 암호화하지 않은 커넥션을 사용한다.
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .host(&self.host)
            .port(self.port)
            .ssl_mode(ssl_mod)
    }
    fn connect_options_with_db(&self) -> PgConnectOptions {
        self.connect_options_without_db()
            .database(&self.database_name)
        // ``.log_statements`은 대한 부분은 저자의 예시 코드에도 보이지 않는다.
        // https://github.com/LukeMathWalker/zero-to-production/blob/root-chapter-05/src/configuration.rs
        // 노이즈를 줄이려고 INFO를 TRACE로 변경하는 것이 이해가 되지 않는다.
    }
}
