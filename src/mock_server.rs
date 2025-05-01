use http::{HeaderMap, Method};
use reqwest::Client;
use serde_json::json;

use crate::configuration::Settings;

pub struct PMMockServer {
    client: Client,
    addr: String,
}

#[derive(serde::Deserialize)]
pub struct PMDebugGet {
    #[serde(with = "http_serde::header_map")]
    header: HeaderMap,
    #[serde(with = "http_serde::method")]
    method: Method,
    body: PMBody,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PMBody {
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "HtmlBody")]
    html_body: String,
    #[serde(rename = "TextBody")]
    text_body: String,
}

#[derive(serde::Deserialize)]
pub struct PMResponse {
    #[serde(rename = "MessageID")]
    message_id: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "SubmittedAt")]
    submitted_at: String,
    #[serde(rename = "ErrorCode")]
    error_code: u32,
    #[serde(rename = "Message")]
    message: String,
}

impl PMMockServer {
    pub fn new(configuration: &Settings) -> PMMockServer {
        PMMockServer {
            client: Client::new(),
            addr: configuration.email_client.base_url.clone(),
        }
    }

    pub async fn health_check(&self) -> Result<reqwest::StatusCode, reqwest::Error> {
        let stauts = self.client.get(&self.addr).send().await?.status();

        Ok(stauts)
    }

    pub async fn send_email(&self, body: &PMBody) -> Result<PMResponse, reqwest::Error> {
        let response = self
            .client
            .post(&format!("http://{}/email", self.addr))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?
            .json::<PMResponse>()
            .await?;

        Ok(response)
    }

    pub async fn check_email(&self, uuid: uuid::Uuid) -> Result<PMDebugGet, reqwest::Error> {
        let response = self
            .client
            .post(&format!("http://{}/email/debug", self.addr))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json!({"uuid": uuid.to_string(), "command": "get"}))
            .send()
            .await?
            .json::<PMDebugGet>()
            .await?;

        Ok(response)
    }
}
