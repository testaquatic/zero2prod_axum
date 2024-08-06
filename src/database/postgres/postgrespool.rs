use futures_util::TryFutureExt;
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgQueryResult, PgSslMode},
    PgPool, Postgres,
};
use uuid::Uuid;

use crate::{
    database::{
        base::{Z2PADBError, Z2PADB},
        ConfirmedSubscriber, UserCredential,
    },
    domain::NewSubscriber,
    settings::DatabaseSettings,
    utils::SubscriptionToken,
};

use super::query::{
    pg_change_password, pg_confirm_subscriber, pg_get_confirmed_subscribers, pg_get_saved_response,
    pg_get_subscriber_id_from_token, pg_get_user_credential, pg_get_user_id, pg_get_username,
    pg_insert_subscriber, pg_store_token,
};

#[derive(Clone)]
pub struct PostgresPool {
    pool: PgPool,
}

impl Z2PADB for PostgresPool {
    type Z2PADBPool = Self;
    type DB = Postgres;

    #[tracing::instrument(name = "Connect to the Postgres server.", skip_all)]
    fn connect(database_settings: &crate::settings::DatabaseSettings) -> Result<Self, Z2PADBError> {
        let pg_connect_options = database_settings.connect_options_with_db();
        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(pg_connect_options);
        Ok(PostgresPool { pool })
    }

    #[tracing::instrument(
        name = "Saving new subscriber details and token in the database."
        skip_all,
    )]
    async fn insert_subscriber(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> Result<SubscriptionToken, Z2PADBError> {
        let mut pg_transaction = self.pool.begin().await.map_err(Z2PADBError::SqlxError)?;
        let subscriber_id = pg_insert_subscriber(
            pg_transaction.as_mut(),
            // 이제 `as_ref`를 사용한다.
            new_subscriber.email.as_ref(),
            new_subscriber.name.as_ref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            Z2PADBError::SqlxError(e)
        })?;

        let subscription_token = SubscriptionToken::generate_subscription_token();
        pg_store_token(
            pg_transaction.as_mut(),
            subscriber_id,
            subscription_token.as_ref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            Z2PADBError::SqlxError(e)
        })?;

        pg_transaction
            .commit()
            .await
            .map_err(Z2PADBError::SqlxError)?;

        Ok(subscription_token)
    }

    #[tracing::instrument(name = "Mark subscriber as confirmed", skip_all)]
    async fn confirm_subscriber(&self, subscriber_id: Uuid) -> Result<PgQueryResult, Z2PADBError> {
        pg_confirm_subscriber(self.as_ref(), subscriber_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                Z2PADBError::SqlxError(e)
            })
    }

    #[tracing::instrument(name = "Get subscriber_id from token", skip_all)]
    async fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> Result<Option<Uuid>, Z2PADBError> {
        pg_get_subscriber_id_from_token(self.as_ref(), subscription_token)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                Z2PADBError::SqlxError(e)
            })
    }

    #[tracing::instrument(name = "Get confirmed subscribers", skip_all)]
    async fn get_confirmed_subscribers(&self) -> Result<Vec<ConfirmedSubscriber>, Z2PADBError> {
        pg_get_confirmed_subscribers(self.as_ref())
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    async fn get_user_id(
        &self,
        username: &str,
        password_hash: Secret<String>,
    ) -> Result<Option<Uuid>, Z2PADBError> {
        pg_get_user_id(self.as_ref(), username, password_hash)
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    #[tracing::instrument(name = "Get stored credentials", skip_all)]
    async fn get_user_credentials(
        &self,
        username: &str,
    ) -> Result<Option<UserCredential>, Z2PADBError> {
        pg_get_user_credential(self.as_ref(), username)
            .map_err(Z2PADBError::SqlxError)
            .await
    }

    #[tracing::instrument(name = "Get username", skip(self))]
    async fn get_username(&self, user_id: Uuid) -> Result<String, Z2PADBError> {
        pg_get_username(self.as_ref(), user_id)
            .map_err(Z2PADBError::SqlxError)
            .await
    }

    async fn change_password(
        &self,
        user_id: Uuid,
        password_hash: Secret<String>,
    ) -> Result<PgQueryResult, Z2PADBError> {
        pg_change_password(self.as_ref(), user_id, password_hash)
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    async fn get_saved_response(
        &self,
        idempotency_key: &crate::idempotency::IdempotencyKey,
        user_id: Uuid,
    ) -> Result<Option<crate::database::base::SavedHttpResponse>, sqlx::Error> {
        pg_get_saved_response(self.as_ref(), idempotency_key.as_ref(), user_id).await
    }
}

impl AsRef<PgPool> for PostgresPool {
    fn as_ref(&self) -> &PgPool {
        // 호출자는 inner 문자열에 대한 공유 참조를 얻는다.
        // 호출자는 읽기 전용으로 접근할 수 있으며, 이는 불변량을 깨뜨리지 못한다.
        &self.pool
    }
}

impl TryFrom<PostgresPool> for PgPool {
    type Error = String;
    fn try_from(value: PostgresPool) -> Result<Self, Self::Error> {
        Ok(value.pool)
    }
}

impl PostgresPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub trait DatabaseSettingsPgExt {
    fn connect_options_without_db(&self) -> PgConnectOptions;
    fn connect_options_with_db(&self) -> PgConnectOptions;
}

impl DatabaseSettingsPgExt for DatabaseSettings {
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
