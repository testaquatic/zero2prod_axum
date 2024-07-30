use crate::{
    domain::NewSubscriber,
    settings::DatabaseSettings,
    utils::{error_chain_fmt, SubscriptionToken},
};
use secrecy::Secret;
use sqlx::{Database, Pool};
use uuid::Uuid;

// DB 변경을 쉽게 하기 위한 트레이트
#[derive(thiserror::Error)]
pub enum Z2PADBError {
    #[error("Z2PADB: Store Token Error")]
    StoreTokenError(#[source] sqlx::Error),
    #[error("Z2PADB: Pool Error")]
    PoolError(#[source] sqlx::Error),
    #[error("Z2PADB: Insert Subscriber Error")]
    InsertSubscriberError(#[source] sqlx::Error),
    #[error("Z2PADB: Transaction Error")]
    TransactionError(#[source] sqlx::Error),
    // 그밖의 자세히 구분할 필요가 없는 오류들
    #[error(transparent)]
    SqlxError(sqlx::Error),
}

impl std::fmt::Debug for Z2PADBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(sqlx::FromRow)]
pub struct ConfirmedSubscriber {
    pub email: String,
}

pub struct UserCredential {
    pub user_id: Uuid,
    pub password_hash: Secret<String>,
}

#[trait_variant::make(Send)]
pub trait Z2PADB: AsRef<Pool<Self::DB>> + Sized {
    type Z2PADBPool: Z2PADB<DB = Self::DB>;
    type DB: Database;
    fn connect(database_settings: &DatabaseSettings) -> Result<Self::Z2PADBPool, Z2PADBError>;

    /// 반환 값은 구독자의 uuid이다.
    async fn insert_subscriber(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> Result<SubscriptionToken, Z2PADBError>;

    async fn confirm_subscriber(
        &self,
        subscriber_id: Uuid,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> Result<Option<Uuid>, Z2PADBError>;

    async fn get_confirmed_subscribers(&self) -> Result<Vec<ConfirmedSubscriber>, Z2PADBError>;

    async fn validate_credentials(
        &self,
        username: &str,
        password_hash: Secret<String>,
    ) -> Result<Option<Uuid>, Z2PADBError>;

    async fn get_user_credentials(
        &self,
        username: &str,
    ) -> Result<Option<UserCredential>, Z2PADBError>;
}
