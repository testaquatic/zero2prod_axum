use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::{domain::SubscriberEmail, error::EmailClientError};

pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    // 우발적인 로깅을 원치 않는다.
    authorization_token: Secret<String>,
}

// 라이프타임 파라미터는 항상 `'`으로 시작한다.
#[derive(serde::Serialize)]
// #[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    // 필드마다 파스칼 케이스로 지정했다.
    #[serde(rename = "From")]
    from: &'a str,
    #[serde(rename = "To")]
    to: &'a str,
    #[serde(rename = "Subject")]
    subject: &'a str,
    #[serde(rename = "HtmlBody")]
    html_body: &'a str,
    #[serde(rename = "TextBody")]
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: &str,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Result<Self, EmailClientError> {
        let email_client = Self {
            http_client: Client::new(),
            base_url: reqwest::Url::parse(&base_url)?,
            sender,
            authorization_token,
        };

        Ok(email_client)
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
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use fake::{
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
        Fake, Faker,
    };
    use secrecy::Secret;
    use serde_json::Value;
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            // body를 JSON 값으로 파싱한다.
            let result = serde_json::from_slice::<Value>(&request.body);
            if let Ok(body) = result {
                // 필드값을 조사하지 않고, 모든 함수 필드들이 입력되었는지 확인한다.
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // 파싱이 실패하면 요청을 매칭하지 않는다.
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // 준비
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::try_from(SafeEmail().fake::<String>()).unwrap();
        let authorization_token = Secret::new(Faker.fake());
        let email_client =
            EmailClient::new(&mock_server.uri(), sender, authorization_token).unwrap();

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // 커스텀 matcher를 사용한다.
            .and(SendEmailBodyMatcher)
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
        // mock 기댓값은 해제 시 체크한다.
    }
}
