use crate::{
    domain::{NewSubscriber, SubscriberEmail},
    error::Zero2ProdAxumError,
    settings::EmailClientSettings,
    utils::SubscriptionToken,
};

// 이메일 전송 서비스 변경시 코드 재사용을 위한 트레이트
#[trait_variant::make(Send)]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), Zero2ProdAxumError>;

    fn from_email_client_settings(
        email_client_settings: &EmailClientSettings,
    ) -> Result<Self, Zero2ProdAxumError>
    where
        Self: Sized;

    #[tracing::instrument(name = "Send a confirmation email to a new subscriber.", skip_all)]
    fn send_confirmation_email(
        &self,
        new_subscriber: NewSubscriber,
        base_url: &str,
        subscription_token: &SubscriptionToken,
    ) -> impl std::future::Future<Output = Result<(), Zero2ProdAxumError>> + Send
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
            self.send_email(new_subscriber.email, "Welcome", &html_body, &text_body)
                .await
        }
    }
}
