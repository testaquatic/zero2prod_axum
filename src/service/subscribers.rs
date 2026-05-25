use rand::distr::SampleString;
use sqlx::PgPool;

use crate::{
    database::postgres::{
        self,
        confirm::{confirm_subscriber, get_subscriber_id_from_token},
        subscription,
    },
    domain::new_subscriber::NewSubscriber,
    email_client,
    handler::subscriptions::SubscribeFormData,
    service::error::ServiceError,
};

pub struct SubscribersService {
    pg_pool: PgPool,
}

impl SubscribersService {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }

    /// 구독 요청을 처리한다.
    pub async fn subscribe(
        &self,
        email_client: &email_client::EmailClient,
        subscriber_form: SubscribeFormData,
        base_url: &str,
    ) -> Result<(), ServiceError> {
        let new_subscriber =
            NewSubscriber::try_from(subscriber_form).map_err(ServiceError::ValidationError)?;

        let mut transaction = self.pg_pool.begin().await?;
        let subscriber_id =
            postgres::subscription::insert_subscriber(transaction.as_mut(), &new_subscriber)
                .await?;
        let subscription_token = generate_subscription_token();

        subscription::store_token(transaction.as_mut(), subscriber_id, &subscription_token).await?;

        transaction.commit().await?;

        send_confirmation_email(email_client, &new_subscriber, base_url, &subscription_token)
            .await?;

        Ok(())
    }

    /// 구독을 확인한다.
    pub async fn confirm(&self, subscription_token: &str) -> Result<(), ServiceError> {
        let id = get_subscriber_id_from_token(&self.pg_pool, subscription_token)
            .await?
            .ok_or_else(|| ServiceError::SubscriptionTokenError)?;

        confirm_subscriber(&self.pg_pool, id).await?;

        Ok(())
    }
}

/// 구독 확인 이메일 전송한다.
#[tracing::instrument(name = "Sending a confirmation email to a new subscriber", skip_all)]
async fn send_confirmation_email(
    email_client: &email_client::EmailClient,
    new_subscriber: &NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), ServiceError> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        r#"Welcome to our newsletter!
Visit {} to confirm your subscription."#,
        confirmation_link
    );
    let html_body = format!(
        r#"Welcome to our newsletter<br />
Click <a href="{}">here</a> to confirm your subscription."#,
        confirmation_link
    );

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &plain_body, &html_body)
        .await?;

    Ok(())
}

/// 대소문자를 구분하는 25자의 구독 토큰을 생성한다.
/// https://docs.rs/rand/latest/rand/distr/struct.Alphanumeric.html
pub fn generate_subscription_token() -> String {
    let mut rng = rand::rng();
    rand::distr::Alphanumeric.sample_string(&mut rng, 25)
}
