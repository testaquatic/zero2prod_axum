use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};

use crate::domain::subscriber_email::SubscriberEmail;

/// 이메일을 전송한다.
pub struct EmailClient {
    /// HTTP Client
    http_client: reqwest::Client,
    /// 요청을 만들 API의 URL
    base_url: String,
    /// 발신자의 이메일 주소
    sender: SubscriberEmail,
    // 인증 토큰
    authorization_token: SecretString,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendEmailRequest<'a> {
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
        authorization_token: SecretString,
    ) -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build http client"),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body,
            text_body,
        };

        self.http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use fake::{
        Fake, Faker,
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
    };
    use reqwest::{StatusCode, header};
    use secrecy::SecretString;
    use wiremock::{
        Mock, MockServer, ResponseTemplate, http,
        matchers::{self},
    };

    use crate::{domain::subscriber_email::SubscriberEmail, email_client::EmailClient};

    /// 커스텀 matcher
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result = serde_json::from_slice::<serde_json::Value>(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() -> Result<(), anyhow::Error> {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).map_err(|s| anyhow::anyhow!(s))?;
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretString::new(Faker.fake::<String>().into()),
        );

        Mock::given(matchers::header_exists("X-Postmark-Server-Token"))
            .and(matchers::header(header::CONTENT_TYPE, "application/json"))
            .and(matchers::path("/email"))
            .and(matchers::method(http::Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Sentence(1..10).fake::<String>();

        let _ = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        Ok(())
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() -> Result<(), anyhow::Error> {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).map_err(|s| anyhow::anyhow!(s))?;
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretString::new(Faker.fake::<String>().into()),
        );

        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Sentence(1..10).fake::<String>();

        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        assert!(outcome.is_ok(), "expected Ok but got {outcome:?}");

        Ok(())
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() -> Result<(), anyhow::Error> {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).map_err(|s| anyhow::anyhow!(s))?;
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretString::new(Faker.fake::<String>().into()),
        );

        Mock::given(matchers::any())
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Sentence(1..10).fake::<String>();

        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        assert!(outcome.is_err(), "expected Err but got {outcome:?}");

        Ok(())
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() -> Result<(), anyhow::Error> {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).map_err(|s| anyhow::anyhow!(s))?;
        let email_client = EmailClient::new(
            mock_server.uri(),
            sender,
            SecretString::new(Faker.fake::<String>().into()),
        );

        let response = ResponseTemplate::new(StatusCode::OK).set_delay(Duration::from_secs(180));

        Mock::given(matchers::any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Sentence(1..10).fake::<String>();

        let outcome = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        assert!(outcome.is_err(), "expected Err but got {outcome:?}");

        Ok(())
    }
}
