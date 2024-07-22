use reqwest::Client;
use secrecy::Secret;

use crate::{domain::SubscriberEmail, error::EmailClientError};

pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    // 우발적인 로깅을 원치 않는다.
    authorization_token: Secret<String>,
}

#[derive(serde::Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Result<Self, EmailClientError> {
        let base_url = Self {
            http_client: Client::new(),
            base_url: reqwest::Url::parse(&base_url)?,
            sender,
            authorization_token,
        };
        Ok(base_url)
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        // `base_url`의 타입을 `String`에서 `reqwest::Url`로 변경하면 `reqwest::Url::join`을 사용해서 더 나은 구현을 할 수 있다.
        let url = self.base_url.join("/email")?;
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        self.http_client
            .post(url)
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // 준비
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::try_from(SafeEmail().fake::<String>()).unwrap();
        let authorization_token = Secret::new(Faker.fake());
        let email_client =
            EmailClient::new(mock_server.uri(), sender, authorization_token).unwrap();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::try_from(SafeEmail().fake::<String>()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Sentence(1..10).fake::<String>();

        // 실행
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // 확인
    }
}
