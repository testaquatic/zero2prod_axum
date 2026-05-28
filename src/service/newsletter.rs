use crate::{
    app_state::AppState,
    database::postgres::subscriptions::get_confirmed_subscribers,
    domain::{form_data::PostNewsletterFormData, subscriber_email::SubscriberEmail},
    service::error::ServiceError,
};

pub struct NewsletterService;

impl NewsletterService {
    pub async fn publish_newsletter(
        &self,
        app_stat: &AppState,
        email: &PostNewsletterFormData,
    ) -> Result<(), ServiceError> {
        let confirmed_subscribers = get_confirmed_subscribers(&app_stat.pg_pool).await?;
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
            app_stat
                .email_client
                .send_email(
                    &subscriber,
                    &email.title,
                    &email.html_content,
                    &email.text_content,
                )
                .await
                .inspect_err(|e| {
                    tracing::error!(e = ?e, "Failed to send newsletter issue to {}", subscriber);
                })?;
        }

        Ok(())
    }
}
