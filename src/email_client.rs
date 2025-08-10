use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

/// 이메일 전송을 담당하는 타입이다.
/// Postmark의 서비스를 이용한다.
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    /// `EmailClient`를 생성한다.
    /// 타임아웃은 10초이다.
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    /// 이메일을 전송한다.
    /// - method
    ///   POST
    /// - header
    ///   Accept: application/json
    ///   Content-Type: application/json
    ///   X-Postmark-Server-Token: server token
    /// - body
    ///   아래의 [SendEmailRequest]를 참조한다.
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
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

/// 이메일 전송 요청을 할 요청본문에 들어가는 json을 모델링한다.
/// json의 예는 다음과 같다
/// ```json
/// {
///     "From": "sender@example.com",
///     "To": "receiver@example.com",
///     "Subject": "Subject",
///     "TextBody": "TextBody",
///     "HtmlBody": "<html><body>HtmlBody</body></html>"
/// }
/// ```
/// 필드별로 이름을 지정하는 방법을 검색해보니
/// [Container attributes](https://serde.rs/container-attrs.html)를 찾을 수 있었다.
#[derive(serde::Serialize)]
struct SendEmailRequest<'a> {
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

#[cfg(test)]
mod tests {

    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use reqwest::{Method, StatusCode, header};
    use secrecy::SecretString;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{any, header, header_exists, method, path},
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    /// 무작위로 이메일 제목을 생성한다.
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// 무작위로 이메일 내용을 생성한다.
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// 무작위로 구독자 이메일을 생성한다.
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// `EmailClient`의 테스트 인스턴스를 얻는다.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretString::from(Faker.fake::<String>()),
            // 테스트에서는 긴 타임아웃이 필요없다.
            std::time::Duration::from_millis(200),
        )
    }

    /// anwkrd

    /// 요청 바디에 필수 필드들이 있는지 확인한다.
    /// 바디는 [super::SendEmailRequest]과 같은 형식이다.
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            serde_json::from_slice::<serde_json::Value>(&request.body)
                .map(|body| {
                    body.get("From").is_some()
                        && body.get("To").is_some()
                        && body.get("Subject").is_some()
                        && body.get("HtmlBody").is_some()
                        && body.get("TextBody").is_some()
                })
                .unwrap_or(false)
        }
    }

    /// 이메일 전송 요청이 유효한지 확인한다.
    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // 준비
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 이 서버는
        // 1. "X-Postmark-Server-Token" 헤더가 있는지
        // 2. "Content-Type"헤더가 "application/json" 인지
        // 3. 경로가 "/email"인지
        // 4. 메소드가 POST인지
        // 5. 요청 바디에 필수적인 필드가 있는지
        // 6. 요청은 한번만 들어왔는지
        // 확인하고
        // 200 OK를 반환한다.
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header(header::CONTENT_TYPE, "application/json"))
            .and(path("/email"))
            .and(method(Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        // 실행
        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        // mock_server가 범위에서 벗어나면 확인이 이루어 진다.
    }

    /// 서버가 200 OK를 반환하면 결과는 Ok(_)이다.
    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // 준비
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 이 이메일 서버는 하나의 요청에 대해서 200 OK를 반환한다.
        Mock::given(any())
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        // 실행
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        claim::assert_ok!(outcome);
    }

    /// 서버가 500 Internal Server Error를 반환하면 `EmailClient::send_email`은 Err(_)를 반환해야 한다.
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // 준비
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 이 이메일 서버는 하나의 요청에 대해서 500 Internal Server Error를 반환한다.
        Mock::given(any())
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .expect(1)
            .mount(&mock_server)
            .await;

        // 실행
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        claim::assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // 준비
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // 이 이메일 서버는 하나의 요청에 대해서 200 OK를 180초 후에 반환한다.
        let response =
            ResponseTemplate::new(StatusCode::OK).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // 실행
        let coutcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // 확인
        claim::assert_err!(coutcome);
    }
}
