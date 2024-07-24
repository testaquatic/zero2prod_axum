use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::{
    domain::{NewSubscriber, SubscriberEmail},
    error::Zero2ProdAxumError,
    settings::EmailClientSettings,
    utils::SubscriptionToken,
};

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
        timeout: std::time::Duration,
    ) -> Result<Self, Zero2ProdAxumError> {
        let http_client = Client::builder().timeout(timeout).build()?;
        let email_client = Self {
            http_client,
            base_url: reqwest::Url::parse(base_url)?,
            sender,
            authorization_token,
        };

        Ok(email_client)
    }

    pub fn from_email_client_settings(
        email_client_settings: &EmailClientSettings,
    ) -> Result<Self, Zero2ProdAxumError> {
        let base_url = &email_client_settings.base_url;
        let sender = email_client_settings.get_sender_email()?;
        let authorization_token = email_client_settings.authorization_token.clone();
        let timeout = std::time::Duration::from_micros(email_client_settings.timeout_milliseconds);

        EmailClient::new(base_url, sender, authorization_token, timeout)
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), Zero2ProdAxumError> {
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
            .await?
            .error_for_status()?;
        Ok(())
    }

    #[tracing::instrument(name = "Send a confirmation email to a new subscriber.", skip_all)]
    pub async fn send_confirmation_email(
        &self,
        new_subscriber: NewSubscriber,
        base_url: &str,
        subscription_token: &SubscriptionToken,
    ) -> Result<(), Zero2ProdAxumError> {
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

#[cfg(test)]
mod tests {

    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use claim::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use serde_json::Value;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    // 여기에서는 필요 없는 것 같지만 tests/ 와 최대한 동일한 코드를 적용했다.
    pub struct TestEmailServer {
        mock_server: MockServer,
    }

    impl TestEmailServer {
        async fn new() -> Self {
            Self {
                mock_server: MockServer::start().await,
            }
        }

        fn uri(&self) -> String {
            self.mock_server.uri()
        }

        pub async fn test_run(&self, mock: Mock) {
            mock.mount(&self.mock_server).await
        }
    }

    // 무작위로 이메일 제목을 생성한다.
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    // 무작위로 이메일 내용을 생성한다.
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    // 무작위로 구독자 이메일을 생성한다.
    fn email() -> SubscriberEmail {
        SubscriberEmail::try_from(SafeEmail().fake::<String>()).unwrap()
    }

    // `EmailClient`의 테스트 인스턴스를 얻는다.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            &base_url,
            email(),
            Secret::new(Faker.fake()),
            // 10초보다 훨씬 짧다.
            std::time::Duration::from_millis(200),
        )
        .unwrap()
    }

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
        // 테스트 데이터
        let subject = subject();
        let content = content();
        let mock = Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // 커스텀 matcher를 사용한다.
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1);

        // 준비
        let test_email_server = TestEmailServer::new().await;

        // 실행
        test_email_server.test_run(mock).await;
        let email_client = email_client(test_email_server.uri());
        let _ = email_client
            .send_email(email(), &subject, &content, &content)
            .await;

        // 확인
        // mock 기댓값은 해제 시 체크한다.
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // 준비
        let test_email_server = TestEmailServer::new().await;
        // 다른 테스트에 있는 모든 매처를 복사하지 않는다.
        // 이 테스트의 목적은 밖으로 내보내는 요청에 대한 어서션을 하지 않는 것이다.
        // `send_email`에서 테스트 하기 위한 경로를 트리거 하기 위한 최소한의 것만 추가한다.
        let mock = Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1);

        // 실행
        test_email_server.test_run(mock).await;
        let eamil_client = email_client(test_email_server.uri());
        let outcome = eamil_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // 준비
        let test_email_server = TestEmailServer::new().await;
        let mock = Mock::given(any())
            // 더 이상 200이 아니다.
            .respond_with(ResponseTemplate::new(500))
            .expect(1);

        // 실행
        test_email_server.test_run(mock).await;
        let email_client = email_client(test_email_server.uri());
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // 준비
        let test_email_server = TestEmailServer::new().await;
        let response = ResponseTemplate::new(200)
            // 3분!
            .set_delay(std::time::Duration::from_secs(180));
        let mock = Mock::given(any()).respond_with(response).expect(1);

        // 실행
        test_email_server.test_run(mock).await;
        let email_client = email_client(test_email_server.uri());
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        assert_err!(outcome);
    }
}
