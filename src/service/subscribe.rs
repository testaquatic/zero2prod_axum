use crate::{
    database::postgres::{self, PostgresDatabase},
    domain::new_subscriber::NewSubscriber,
    email_client,
    handler::subscriptions::SubscribeFormData,
    service::error::ServiceError,
};

pub struct SubscribeService {
    postgres_database: PostgresDatabase,
}

impl SubscribeService {
    pub fn new(postgres_database: PostgresDatabase) -> Self {
        Self { postgres_database }
    }

    /// 구독 요청을 처리한다.
    pub async fn subscribe(
        &self,
        email_client: &email_client::EmailClient,
        subscriber_form: SubscribeFormData,
        base_url: &str,
    ) -> Result<(), ServiceError> {
        let new_subscriber = NewSubscriber::try_from(subscriber_form)
            .map_err(|e| ServiceError::ValidationError(e))?;
        postgres::subscription::insert_subscriber(&self.postgres_database, &new_subscriber).await?;

        send_confirmation_email(email_client, &new_subscriber, base_url).await?;

        Ok(())
    }
}

/// 구독 확인 이메일 전송한다.
#[tracing::instrument(name = "Sending a confirmation email to a new subscriber", skip_all)]
async fn send_confirmation_email(
    email_client: &email_client::EmailClient,
    new_subscriber: &NewSubscriber,
    base_url: &str,
) -> Result<(), ServiceError> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token=my_token",
        base_url
    );
    let plain_body = format!(
        r#"Welcome to our newsletter!
Visit {} to confirm your subscription."#,
        confirmation_link
    );
    let html_body = &format!(
        r#"Welcome to our newsletter<br />
Click <a href="{}">here</a> to confirm your subscription."#,
        confirmation_link
    );

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &plain_body, &html_body)
        .await?;

    Ok(())
}
