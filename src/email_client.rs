use reqwest::{Client, Response};
use secrecy::{ExposeSecret, SecretString};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        timeout: std::time::Duration,
    ) -> Result<Self, reqwest::Error> {
        let http_client = Client::builder().timeout(timeout).build()?;
        let email_client = EmailClient {
            http_client,
            base_url,
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
    ) -> Result<Response, reqwest::Error> {
        let url = format!("http://{}/email", self.base_url);
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
            .error_for_status()
    }
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

#[cfg(test)]
mod tests {

    use claim::{assert_err, assert_ok};
    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use pm_mock_server::PMMockServer;
    use reqwest::StatusCode;
    use secrecy::SecretString;
    use serde_json::Value;

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
    fn email() -> Result<SubscriberEmail, anyhow::Error> {
        SubscriberEmail::try_from(SafeEmail().fake::<String>()).map_err(|e| anyhow::anyhow!(e))
    }

    /// `EmailClient`의 테스트 인스턴스를 얻는다.
    fn email_client(base_url: String) -> Result<EmailClient, anyhow::Error> {
        let email_client = EmailClient::new(
            base_url,
            email()?,
            SecretString::new(Faker.fake::<String>().into_boxed_str()),
            std::time::Duration::from_millis(200),
        )?;

        Ok(email_client)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_email_sends_the_expected_request() -> Result<(), anyhow::Error> {
        // 준비
        let pm_mock_server = PMMockServer::start_server().await?;
        let email_client = email_client(pm_mock_server.addr.clone())?;

        // 실행
        let response = email_client
            .send_email(email()?, &subject(), &content(), &content())
            .await?;

        // 확인
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.json::<serde_json::Map<String, Value>>().await?;
        let uuid = body
            .get("MessageID")
            .expect("No MessageID in response")
            .as_str()
            .expect("Failed to get MessageID");

        let debug = pm_mock_server.get_request_info(&uuid).await?;
        debug
            .header_exists("X-Postmark-Server-Token")
            .header_check(reqwest::header::CONTENT_TYPE, "application/json")
            .method_check(reqwest::Method::POST);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_email_succeeds_if_the_server_returns_200() -> Result<(), anyhow::Error> {
        // 준비
        let pm_mock_server = PMMockServer::start_server().await?;
        let email_client = email_client(pm_mock_server.addr.clone())?;

        // 실행
        let outcome = email_client
            .send_email(email()?, &subject(), &content(), &content())
            .await;

        // 확인
        assert_ok!(outcome);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_email_fails_if_the_server_returns_500() -> Result<(), anyhow::Error> {
        // 준비
        let pm_mock_server = PMMockServer::start_server().await?;
        let email_client = email_client(format!("{}/500", pm_mock_server.addr))?;

        // 실행
        let coutcome = email_client
            .send_email(email()?, &subject(), &content(), &content())
            .await;

        // 확인
        assert_err!(coutcome);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn send_email_times_out_if_the_server_takes_too_long() -> Result<(), anyhow::Error> {
        // 준비
        let pm_mock_server = PMMockServer::start_server().await?;
        let email_client = email_client(format!("{}/delay", pm_mock_server.addr))?;

        // 실행
        let outcome = email_client
            .send_email(email()?, &subject(), &content(), &content())
            .await;

        // 확인
        assert_err!(outcome);

        Ok(())
    }
}
