use crate::{domain::NewSubscriber, settings::DatabaseSettings, utils::SubscriptionToken};
use sqlx::{Database, Pool};
use uuid::Uuid;

#[trait_variant::make(Send)]
pub trait Zero2ProdAxumDatabase: AsRef<Pool<Self::DB>> + Sized {
    type Z2PADBPool: Zero2ProdAxumDatabase<DB = Self::DB>;
    type DB: Database;
    fn connect(database_settings: &DatabaseSettings) -> Result<Self::Z2PADBPool, sqlx::Error>;

    /// 반환 값은 구독자의 uuid이다.
    async fn insert_subscriber(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> Result<SubscriptionToken, sqlx::Error>;

    async fn confirm_subscriber(
        &self,
        subscriber_id: Uuid,
    ) -> Result<<Self::DB as Database>::QueryResult, sqlx::Error>;

    async fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> Result<Option<Uuid>, sqlx::Error>;
}
