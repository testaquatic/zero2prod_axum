use rand::distr::SampleString;

use crate::{
    app_state::AppState,
    database::postgres::{
        self,
        subscription_tokens::{get_subscriber_id_from_token, save_subscription_token},
        subscriptions::update_subscriber_confirmed,
    },
    domain::{form_data::SubscribeFormData, new_subscriber::NewSubscriber},
    email_client::{self},
    service::error::ServiceError,
};

pub struct SubscriptionsService;

impl SubscriptionsService {
    /// 구독 요청을 처리한다.
    pub async fn subscribe(
        &self,
        app_state: &AppState,
        subscriber_form: SubscribeFormData,
    ) -> Result<(), ServiceError> {
        let new_subscriber =
            NewSubscriber::try_from(subscriber_form).map_err(ServiceError::ValidationError)?;

        let mut transaction = app_state.pg_pool.begin().await?;
        let subscriber_id =
            postgres::subscriptions::save_subscriber(transaction.as_mut(), &new_subscriber).await?;
        let subscription_token = generate_subscription_token();

        save_subscription_token(transaction.as_mut(), subscriber_id, &subscription_token).await?;

        transaction.commit().await?;

        send_confirmation_email(
            &app_state.email_client,
            &new_subscriber,
            app_state.base_url.0.as_str(),
            &subscription_token,
        )
        .await?;

        Ok(())
    }

    /// 구독을 확인한다.
    pub async fn confirm(
        &self,
        app_state: &AppState,
        subscription_token: &str,
    ) -> Result<(), ServiceError> {
        let mut transaction = app_state.pg_pool.begin().await?;
        let id = get_subscriber_id_from_token(transaction.as_mut(), subscription_token)
            .await?
            .ok_or_else(|| ServiceError::SubscriptionTokenError)?;

        update_subscriber_confirmed(transaction.as_mut(), id).await?;
        transaction.commit().await?;

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
