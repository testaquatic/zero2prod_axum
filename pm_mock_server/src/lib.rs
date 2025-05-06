use std::{
    collections::HashMap,
    process::{Command, Stdio},
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};

use http::HeaderMap;
use reqwest::{Client, Response};
use serde_json::json;
use url::Url;
use zero2prod_axum::configuration::{self, EmailClientSettings};

struct PMMockHub {
    client: Client,
    base_url: Url,
    port: u16,
}

pub struct PMMockServer {
    client: Client,
    base_url: Url,
    port: u16,
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

const PORT: u16 = 8800;

impl PMMockHub {
    /// `PMMockHub`를 생성한다.
    fn new(email_client_settings: &EmailClientSettings) -> Result<PMMockHub, url::ParseError> {
        let base_url = Url::parse(&email_client_settings.base_url)?;
        let pm_mock_hub = PMMockHub {
            client: Client::new(),
            base_url,
            port: PORT,
        };

        Ok(pm_mock_hub)
    }

    /// 서버를 생성하고 주소를 반납한다.
    async fn new_server(&self) -> Result<PMMockServer, anyhow::Error> {
        self.hub_server().await?;

        let port = self
            .client
            .get(self.hub_url()?.join("/new_server")?)
            .send()
            .await?
            .text()
            .await?;

        Ok(PMMockServer {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            port: u16::from_str(&port)?,
        })
    }

    fn hub_url(&self) -> Result<Url, anyhow::Error> {
        let mut hub_url = self.base_url.clone();
        hub_url
            .set_port(Some(self.port))
            .map_err(|_| anyhow::anyhow!("Failed to set port"))?;

        Ok(hub_url)
    }

    /// 설정 파일에서 `PMMockServer`를 생성한다.
    async fn new_from_configuration() -> Result<PMMockHub, anyhow::Error> {
        let configuration = configuration::get_configuration()?;
        let pm_mock_server = PMMockHub::new(&configuration.email_client)?;
        // pm_mock_server.is_mock_server_is_on().await;

        Ok(pm_mock_server)
    }

    /// 서버가 작동하는지 테스트한다.
    async fn health_check(&self) -> Result<Response, anyhow::Error> {
        let response = self
            .client
            .get(self.hub_url()?.join("/health_check")?)
            .send()
            .await?
            .error_for_status()?;

        Ok(response)
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
                            .arg("--port")
                            .arg(PORT.to_string())
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
                return Ok(self);
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
    pub async fn get_request_info(&self, uuid: &str) -> Result<PMDebugGet, anyhow::Error> {
        let response = self
            .client
            .get(self.server_url().join("/debug")?)
            .json(&json!({"uuid": uuid, "command": "get"}))
            .send()
            .await?;
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let response = response.json::<PMDebugGet>().await?;

        Ok(response)
    }

    pub async fn get_all_request_infos(
        &self,
    ) -> Result<HashMap<String, PMDebugGet>, anyhow::Error> {
        let response = self
            .client
            .get(self.server_url().join("/debug")?)
            .json(&json!({"command": "get_all"}))
            .send()
            .await?;
        assert_eq!(response.status(), reqwest::StatusCode::OK);

        let response = response.json::<HashMap<String, PMDebugGet>>().await?;

        Ok(response)
    }

    pub async fn recieved_reqeusts(&self) -> Result<Vec<PMBody>, anyhow::Error> {
        let request_infos = self.get_all_request_infos().await?;
        let requests = request_infos
            .into_values()
            .map(|pmdebug_get| pmdebug_get.body)
            .collect();

        Ok(requests)
    }

    /// 비효율적이지만 테스트 코드이니 놔두는 것이 나을 것 같다.
    /// 마지막의 슬래시를 제거해야 코드와 호환된다.
    pub fn url(&self) -> String {
        self.server_url().as_str().trim_end_matches('/').to_string()
    }

    fn server_url(&self) -> Url {
        let mut server_url = self.base_url.clone();
        server_url.set_port(Some(self.port)).unwrap();

        server_url
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
