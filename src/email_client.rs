use reqwest::Client;
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
    ) -> EmailClient {
        EmailClient {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_string(),
            to: recipient.as_ref().to_string(),
            subject: subject.to_string(),
            html_body: html_content.to_string(),
            text_body: text_content.to_string(),
        };
        self.http_client
            .post(&url)
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

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
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
    use secrecy::SecretString;

    use crate::{configuration, domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test(flavor = "multi_thread")]
    async fn send_email_fires_a_request_to_base_url() -> Result<(), anyhow::Error> {
        // 준비
        let sender = SubscriberEmail::try_from(SafeEmail().fake::<String>())
            .map_err(|e| anyhow::anyhow!(e))?;
        let configuration = configuration::get_configuration()?;
        let email_client = EmailClient::new(
            format!("{}/email", configuration.email_client.base_url),
            sender,
            SecretString::from(Faker.fake::<String>()),
        );

        let subscriber_email = SubscriberEmail::try_from(SafeEmail().fake::<String>())
            .map_err(|e| anyhow::anyhow!(e))?;
        let subject = Sentence(1..2).fake::<String>();
        let content = Paragraph(1..10).fake::<String>();

        // 실행
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // 확인

        Ok(())
    }
}
