use std::sync::Arc;

use crate::{
    database::postgres::PostgresDatabase,
    email_client,
    service::{self, subscribe::SubscribeService},
};

pub struct ApplicationBaseUrl(pub String);

pub struct AppState {
    pub subscribe_service: service::subscribe::SubscribeService,
    pub email_client: email_client::EmailClient,
    pub base_url: ApplicationBaseUrl,
}

impl AppState {
    pub fn new(
        pool: sqlx::PgPool,
        email_client: email_client::EmailClient,
        base_url: String,
    ) -> Arc<Self> {
        let app_state = Self {
            subscribe_service: SubscribeService::new(PostgresDatabase::new(pool)),
            email_client,
            base_url: ApplicationBaseUrl(base_url),
        };

        Arc::new(app_state)
    }
}
