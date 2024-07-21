use crate::{domain::NewSubscriber, settings::DatabaseSettings};
use sqlx::{Database, Pool};

#[trait_variant::make(Send)]
pub trait Zero2ProdAxumDatabase: AsRef<Pool<Self::DB>> + Sized {
    type Z2PADBPool: Zero2ProdAxumDatabase<DB = Self::DB>;
    type DB: Database;
    fn connect(database_settings: &DatabaseSettings) -> Result<Self::Z2PADBPool, sqlx::Error>;

    async fn insert_subscriber(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> Result<<Self::DB as Database>::QueryResult, sqlx::Error>;
}
