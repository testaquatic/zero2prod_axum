use secrecy::SecretString;
use sqlx::{PgExecutor, PgPool};

use crate::{
    database::postgres::newsletters::{get_confirmed_subscribers, get_user_id_from_credentials},
    domain::subscriber_email::SubscriberEmail,
    email_client::EmailClient,
    handler::newsletter::BodyData,
    service::error::ServiceError,
};

pub struct NewsletterService;

impl NewsletterService {
    pub async fn publish_newsletter(
        &self,
        pg_pool: &PgPool,
        email: &BodyData,
        email_client: &EmailClient,
    ) -> Result<(), ServiceError> {
        let confirmed_subscribers = get_confirmed_subscribers(pg_pool).await?;
        let subscribers = confirmed_subscribers.into_iter().filter_map(|r| {
            SubscriberEmail::parse(r.email)
                .map_err(|e| {
                    tracing::warn!(
                        error = ?e,
                        "Skipping a confirmed subscriber. Their stored contact details are invalid",
                    )
                })
                .ok()
        });

        for subscriber in subscribers {
            email_client
                .send_email(
                    &subscriber,
                    &email.title,
                    &email.content.html,
                    &email.content.text,
                )
                .await
                .inspect_err(|e| {
                    tracing::error!(e = ?e, "Failed to send newsletter issue to {}", subscriber);
                })?;
        }

        Ok(())
    }

    pub async fn validate_credentials(
        &self,
        pg_executor: impl PgExecutor<'_>,
        username: &str,
        password: &SecretString,
    ) -> Result<uuid::Uuid, ServiceError> {
        let uuid = get_user_id_from_credentials(pg_executor, username, password).await?;

        uuid.ok_or(ServiceError::AuthError)
    }
}
