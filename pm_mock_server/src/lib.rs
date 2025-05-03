use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};

use http::HeaderMap;
use reqwest::{Client, Response};
use serde_json::json;
use zero2prod_axum::configuration::{self, EmailClientSettings};

struct PMMockHub {
    client: Client,
    addr: String,
}

pub struct PMMockServer {
    client: Client,
    pub addr: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct PMDebugGet {
    #[serde(with = "http_serde::header_map")]
    pub header: HeaderMap,
    pub method: String,
    #[serde(rename = "body")]
    pub body: PMBody,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct PMBody {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "Subject")]
    pub subject: String,
    #[serde(rename = "HtmlBody")]
    pub html_body: String,
    #[serde(rename = "TextBody")]
    pub text_body: String,
}

#[derive(serde::Deserialize)]
pub struct PMResponse {
    #[serde(rename = "MessageID")]
    pub message_id: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "SubmittedAt")]
    pub submitted_at: String,
    #[serde(rename = "ErrorCode")]
    pub error_code: u32,
    #[serde(rename = "Message")]
    pub message: String,
}

impl PMMockHub {
    /// `PMMockHub`를 생성한다.
    fn new(email_client_settings: &EmailClientSettings) -> PMMockHub {
        PMMockHub {
            client: Client::new(),
            addr: email_client_settings.base_url.clone(),
        }
    }

    /// 서버를 생성하고 주소를 반납한다.
    async fn new_server(&self) -> Result<PMMockServer, anyhow::Error> {
        self.hub_server().await?;
        let port = self
            .client
            .get(format!("http://{}/new_server", self.addr))
            .send()
            .await?
            .text()
            .await?;

        Ok(PMMockServer {
            client: self.client.clone(),
            addr: format!("localhost:{}", port),
        })
    }

    /// 설정 파일에서 `PMMockServer`를 생성한다.
    async fn new_from_configuration() -> Result<PMMockHub, anyhow::Error> {
        let configuration = configuration::get_configuration()?;
        let pm_mock_server = PMMockHub::new(&configuration.email_client);
        // pm_mock_server.is_mock_server_is_on().await;

        Ok(pm_mock_server)
    }

    /// 서버가 작동하는지 테스트한다.
    async fn health_check(&self) -> Result<Response, reqwest::Error> {
        self.client
            .get(&format!("http://{}/health_check", self.addr))
            .send()
            .await?
            .error_for_status()
    }

    /// 서버를 시작한다.
    async fn hub_server(&self) -> Result<&Self, anyhow::Error> {
        // 서버가 실행중이라면 복잡한 테스트를 할 필요가 없다.
        if self.health_check().await.is_ok() {
            return Ok(self);
        }
        static IS_RUN: AtomicBool = AtomicBool::new(false);
        loop {
            match IS_RUN
                // `Ordering::Relaxed`로도 문제가 없을 듯 하다.
                .compare_exchange_weak(false, true, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => {
                    for _ in 0..5 {
                        Command::new("go")
                            .current_dir("go/mock_server")
                            .arg("run")
                            .arg("main.go")
                            .arg("server.go")
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()?;
                        // 서버의 시작을 기다린다.
                        // 500ms는 지나치게 긴 시간 같긴 하다.
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        if self.health_check().await.is_ok() {
                            break;
                        }
                    }
                }
                Err(true) => break,
                _ => continue,
            }
        }

        for _ in 0..5 {
            if self.health_check().await.is_err() {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            } else {
                return Ok(&self);
            }
        }
        Err(anyhow::anyhow!("Cannot start mock server"))
    }
}

impl PMMockServer {
    pub async fn start_server() -> Result<Self, anyhow::Error> {
        let pm_mock_hub = PMMockHub::new_from_configuration().await?;
        let pm_mock_server = pm_mock_hub.new_server().await?;

        Ok(pm_mock_server)
    }

    /// 이전에 한 요청의 정보를 확인한다.
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

    pub async fn get_all_requests_info(
        &self,
    ) -> Result<HashMap<String, PMDebugGet>, reqwest::Error> {
        let response = self
            .client
            .get(&format!("http://{}/debug", self.addr))
            .json(&json!({"command": "get_all"}))
            .send()
            .await?;
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let response = response.json::<HashMap<String, PMDebugGet>>().await?;

        Ok(response)
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
