use crate::{
    database::{Z2PADBError, Z2PADB},
    domain::{InvalidNewSubscriber, NewSubscriber, SubscriberEmail},
    settings::{DefaultDBPool, EmailClientSettings},
    utils::{error_chain_fmt, SubscriptionToken},
};

#[derive(thiserror::Error)]
pub enum EmailClientError {
    #[error("EmailClient: Url Error")]
    UrlParseError(#[from] url::ParseError),
    #[error("EmailClient: Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("EmailClient: SubscriberEmail Error")]
    SubscriberEmailError(#[from] InvalidNewSubscriber),
    #[error("EmailClient: Z")]
    Z2PADBError(#[from] Z2PADBError),
}

impl std::fmt::Debug for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// 이메일 전송 서비스 변경시 코드 재사용을 위한 트레이트
#[trait_variant::make(Send)]
pub trait EmailClient
where
    Self: Sync + Sized,
{
    async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError>;

    fn from_email_client_settings(
        email_client_settings: &EmailClientSettings,
    ) -> Result<Self, EmailClientError>;

    #[tracing::instrument(name = "Send a confirmation email to a new subscriber.", skip_all)]
    fn send_confirmation_email(
        &self,
        new_subscriber: NewSubscriber,
        base_url: &str,
        subscription_token: &SubscriptionToken,
    ) -> impl std::future::Future<Output = Result<(), EmailClientError>> {
        async move {
            let confirmation_link = format!(
                "{}/subscriptions/confirm?subscription_token={}",
                base_url,
                subscription_token.as_ref()
            );
            let text_body = format!(
                "Welcome to our newletter!\nVisit {} to confirm your subscription",
                confirmation_link
            );
            let html_body = format!(
                "Welcome to our newsletter!<br>
            Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            );
            self.send_email(&new_subscriber.email, "Welcome", &html_body, &text_body)
                .await
        }
    }

    #[tracing::instrument(name = "Publish newsletter", skip_all)]
    async fn publish_newsletter(
        &self,
        pool: &DefaultDBPool,
        body: &BodyData,
    ) -> Result<(), EmailClientError> {
        publish_newsletter(pool, self, body)
    }
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

// 여기에 놓는 것이 맞을 것 같기는 한데 애매하다.
pub async fn publish_newsletter<T: EmailClient>(
    pool: &DefaultDBPool,
    email_client: &T,
    body: &BodyData,
) -> Result<(), EmailClientError> {
    // 이메일을 보낼 구독자 목록을 생성한다.
    let subscribers = pool.get_confirmed_subscribers().await?;

    for subscriber in subscribers {
        let subscriber_email = SubscriberEmail::try_from(subscriber.email)?;
        email_client
            .send_email(
                &subscriber_email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await?;
    }

    Ok(())
}
