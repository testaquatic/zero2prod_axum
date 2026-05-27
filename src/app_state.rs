use std::sync::Arc;

use crate::{
    email_client::{self, EmailClient},
    service::{
        authentication::CredentialService, newsletter::NewsletterService,
        subscriptions::SubscriptionsService,
    },
};

pub struct ApplicationBaseUrl(pub String);

pub struct AppState {
    pub subscribe_service: SubscriptionsService,
    pub newsletter_service: NewsletterService,
    pub credential_service: CredentialService,
    pub pg_pool: sqlx::PgPool,
    pub email_client: EmailClient,
    pub base_url: ApplicationBaseUrl,
}

impl AppState {
    pub fn new(
        pool: sqlx::PgPool,
        email_client: email_client::EmailClient,
        base_url: String,
    ) -> Arc<Self> {
        let app_state = Self {
            subscribe_service: SubscriptionsService,
            newsletter_service: NewsletterService,
            credential_service: CredentialService,
            pg_pool: pool,
            email_client,
            base_url: ApplicationBaseUrl(base_url),
        };

        Arc::new(app_state)
    }
}
