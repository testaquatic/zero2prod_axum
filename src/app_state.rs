use std::sync::Arc;

use crate::{
    database::postgres::PostgresDatabase,
    email_client,
    service::{self, subscribe::SubscribeService},
};

pub struct AppState {
    pub subscribe_service: service::subscribe::SubscribeService,
    pub email_client: email_client::EmailClient,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, email_client: email_client::EmailClient) -> Arc<Self> {
        let app_state = Self {
            subscribe_service: SubscribeService::new(PostgresDatabase::new(pool)),
            email_client,
        };

        Arc::new(app_state)
    }
}
