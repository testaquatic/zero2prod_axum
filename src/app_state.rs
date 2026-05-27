use std::sync::Arc;

use moka::future::Cache;
use secrecy::SecretSlice;
use uuid::Uuid;

use crate::{
    email_client::{self, EmailClient},
    service::{
        auth_token_service::AuthTokenService, authentication::CredentialService,
        newsletter::NewsletterService, subscriptions::SubscriptionsService,
    },
};

pub struct ApplicationBaseUrl(pub String);

pub struct AppState {
    pub subscribe_service: SubscriptionsService,
    pub newsletter_service: NewsletterService,
    pub credential_service: CredentialService,
    pub auth_token_service: AuthTokenService,
    pub pg_pool: sqlx::PgPool,
    pub email_client: EmailClient,
    pub base_url: ApplicationBaseUrl,
    pub moka_cache: Cache<Uuid, Uuid>,
}

impl AppState {
    pub fn new(
        pool: sqlx::PgPool,
        email_client: email_client::EmailClient,
        base_url: String,
        token_secret_private_key: SecretSlice<u8>,
        token_secret_public_key: SecretSlice<u8>,
        moka_cache: Cache<Uuid, Uuid>,
    ) -> Arc<Self> {
        let app_state = Self {
            subscribe_service: SubscriptionsService,
            newsletter_service: NewsletterService,
            credential_service: CredentialService,
            auth_token_service: AuthTokenService {
                token_secret_private_key,
                token_secret_public_key,
            },
            pg_pool: pool,
            email_client,
            base_url: ApplicationBaseUrl(base_url),
            moka_cache,
        };

        Arc::new(app_state)
    }
}
