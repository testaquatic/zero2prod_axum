use futures_util::TryFutureExt;
use secrecy::Secret;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgQueryResult},
    PgPool,
};
use uuid::Uuid;

use crate::{
    database::{types::Z2PADBError, NextAction, UserCredential},
    domain::NewSubscriber,
    utils::SubscriptionToken,
};

use super::{
    postgres_query::{
        pg_change_password, pg_confirm_subscriber, pg_dequeue_task, pg_get_issue,
        pg_get_saved_response, pg_get_subscriber_id_from_token, pg_get_user_credential,
        pg_get_username, pg_insert_subscriber, pg_store_token, pg_try_saving_idempotency_key,
    },
    postgres_transaction::PostgresTransaction,
    types::NewsletterIssue,
};

#[derive(Clone)]
pub struct PostgresPool {
    pool: PgPool,
}

impl AsRef<PgPool> for PostgresPool {
    fn as_ref(&self) -> &PgPool {
        // 호출자는 inner 문자열에 대한 공유 참조를 얻는다.
        // 호출자는 읽기 전용으로 접근할 수 있으며, 이는 불변량을 깨뜨리지 못한다.
        &self.pool
    }
}

impl From<PostgresPool> for PgPool {
    fn from(value: PostgresPool) -> Self {
        value.pool
    }
}

impl PostgresPool {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(name = "Connect to the Postgres server.", skip_all)]
    pub fn connect(pg_connect_options: PgConnectOptions) -> Result<Self, Z2PADBError> {
        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(pg_connect_options);
        Ok(PostgresPool { pool })
    }

    pub async fn begin(&self) -> Result<PostgresTransaction, Z2PADBError> {
        Ok(PostgresTransaction::new(self.pool.begin().await?))
    }

    #[tracing::instrument(
        name = "Saving new subscriber details and token in the database."
        skip_all,
    )]
    pub async fn insert_subscriber(
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
    pub async fn confirm_subscriber(
        &self,
        subscriber_id: Uuid,
    ) -> Result<PgQueryResult, Z2PADBError> {
        pg_confirm_subscriber(self.as_ref(), subscriber_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                Z2PADBError::SqlxError(e)
            })
    }

    #[tracing::instrument(name = "Get subscriber_id from token", skip_all)]
    pub async fn get_subscriber_id_from_token(
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

    #[tracing::instrument(name = "Get stored credentials", skip_all)]
    pub async fn get_user_credentials(
        &self,
        username: &str,
    ) -> Result<Option<UserCredential>, Z2PADBError> {
        pg_get_user_credential(self.as_ref(), username)
            .map_err(Z2PADBError::SqlxError)
            .await
    }

    #[tracing::instrument(name = "Get username", skip(self))]
    pub async fn get_username(&self, user_id: Uuid) -> Result<String, Z2PADBError> {
        pg_get_username(self.as_ref(), user_id)
            .map_err(Z2PADBError::SqlxError)
            .await
    }

    pub async fn change_password(
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
    ) -> Result<Option<crate::database::types::SavedHttpResponse>, Z2PADBError> {
        pg_get_saved_response(self.as_ref(), idempotency_key.as_ref(), user_id)
            .await
            .map_err(Z2PADBError::SqlxError)
    }

    pub async fn try_processing(
        &self,
        idempotency_key: &crate::idempotency::IdempotencyKey,
        user_id: Uuid,
    ) -> Result<NextAction, Z2PADBError> {
        let mut transaction = self.begin().await?;

        match pg_try_saving_idempotency_key(
            transaction.as_mut().as_mut(),
            idempotency_key.as_ref(),
            user_id,
        )
        .await?
        .rows_affected()
        {
            0 => {
                let saved_response = self
                    .get_saved_response(idempotency_key, user_id)
                    .await?
                    .ok_or(Z2PADBError::NoSavedResponse)?;
                Ok(NextAction::ReturnSavedResponse(saved_response))
            }
            _ => Ok(NextAction::StartProcessing(transaction)),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn dequeue_task(
        &self,
    ) -> Result<Option<(PostgresTransaction, Uuid, String)>, Z2PADBError> {
        let mut transaction = self.begin().await?;
        let r = pg_dequeue_task(transaction.as_mut().as_mut())
            .await
            .map_err(Z2PADBError::SqlxError)?;

        match r {
            Some((uuid, email)) => Ok(Some((transaction, uuid, email))),
            None => Ok(None),
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn get_issue(&self, issue_id: Uuid) -> Result<NewsletterIssue, Z2PADBError> {
        pg_get_issue(self.as_ref(), issue_id)
            .await
            .map_err(Z2PADBError::SqlxError)
    }
}
