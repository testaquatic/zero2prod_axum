use claim::assert_ok;
use http::HeaderMap;
use reqwest::{Client, Response};
use serde_json::json;

use crate::configuration::{self, EmailClientSettings};

pub struct PMMockServer {
    client: Client,
    pub addr: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct PMDebugGet {
    #[serde(with = "http_serde::header_map")]
    header: HeaderMap,
    method: String,
    #[serde(rename = "body")]
    _body: PMBody,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
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
    _message_id: String,
    #[serde(rename = "To")]
    _to: String,
    #[serde(rename = "SubmittedAt")]
    _submitted_at: String,
    #[serde(rename = "ErrorCode")]
    _error_code: u32,
    #[serde(rename = "Message")]
    _message: String,
}

impl PMMockServer {
    pub fn new(email_client_settings: &EmailClientSettings) -> PMMockServer {
        PMMockServer {
            client: Client::new(),
            addr: email_client_settings.base_url.clone(),
        }
    }

    pub async fn new_from_configuration() -> Result<PMMockServer, config::ConfigError> {
        let configuration = configuration::get_configuration()?;
        let pm_mock_server = PMMockServer::new(&configuration.email_client);
        pm_mock_server.is_mock_server_is_on().await;

        Ok(PMMockServer::new(&configuration.email_client))
    }

    pub async fn health_check(&self) -> Result<Response, reqwest::Error> {
        self.client
            .get(&format!("http://{}/health_check", self.addr))
            .send()
            .await?
            .error_for_status()
    }

    pub async fn get_request_info(&self, uuid: &str) -> Result<PMDebugGet, reqwest::Error> {
        let response = self
            .client
            .get(&format!("http://{}/debug", self.addr))
            .json(&json!({"uuid": uuid, "command": "get"}))
            .send()
            .await?;
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let response = response.json::<PMDebugGet>().await?;

        Ok(response)
    }

    pub async fn is_mock_server_is_on(&self) -> &Self {
        assert_ok!(
            self.health_check()
                .await
                .expect("Cannot connect to mock server. Run mock server first.")
                .error_for_status()
        );

        self
    }
}

impl PMDebugGet {
    pub fn header_exists(&self, key: &str) -> &Self {
        assert!(
            self.header.contains_key(key),
            "Header does not exist: {}",
            key
        );

        self
    }

    pub fn header_check(&self, key: impl AsRef<str>, value: &str) -> &Self {
        assert_eq!(
            self.header.get(key.as_ref()).expect("No header found"),
            value,
            "Header value does not match: {}",
            key.as_ref()
        );

        self
    }

    pub fn method_check(&self, method: impl AsRef<str>) -> &Self {
        assert_eq!(
            self.method.to_string(),
            method.as_ref(),
            "Method does not match: {}",
            method.as_ref()
        );

        self
    }
}
