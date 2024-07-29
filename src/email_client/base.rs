use crate::{
    domain::{NewSubscriber, SubscriberEmail},
    settings::EmailClientSettings,
    utils::{error_chain_fmt, SubscriptionToken},
};

#[derive(thiserror::Error)]
pub enum EmailClientError {
    #[error("EmailClient: Url Error")]
    UrlParseError(#[from] url::ParseError),
    #[error("EmailClient: Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("EmailClient: SubscriberEmail Error")]
    SubscriberEmailError(String),
}

impl std::fmt::Debug for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// 이메일 전송 서비스 변경시 코드 재사용을 위한 트레이트
#[trait_variant::make(Send)]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError>;

    fn from_email_client_settings(
        email_client_settings: &EmailClientSettings,
    ) -> Result<Self, EmailClientError>
    where
        Self: Sized;

    #[tracing::instrument(name = "Send a confirmation email to a new subscriber.", skip_all)]
    fn send_confirmation_email(
        &self,
        new_subscriber: NewSubscriber,
        base_url: &str,
        subscription_token: &SubscriptionToken,
    ) -> impl std::future::Future<Output = Result<(), EmailClientError>> + Send
    where
        Self: Sync,
    {
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
}
