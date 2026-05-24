use std::sync::Arc;

use crate::{database::postgres, service};

#[derive(Clone)]
pub struct AppState {
    pub subscribe_service: Arc<service::subscribe::SubscribeService>,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            subscribe_service: Arc::new(service::subscribe::SubscribeService::new(
                postgres::PostgresDatabase::new(pool),
            )),
        }
    }
}
