use std::sync::Once;

use anyhow::Context;
use tokio::net::TcpListener;
use tracing::{level_filters::LevelFilter, Subscriber};
use url::Url;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod_axum::{
    database::Z2PADBError,
    error::Z2PAError,
    settings::{DefaultDBPool, Settings},
    startup::Server,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

use super::DefaultDBPoolTestExt;

pub struct TestApp {
    pub settings: Settings,
    pub email_mock_server: MockServer,
}

/// 이메일 API에 대한 요청에 포함된 확인 링크
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    /// 애플리케이션 인스턴스를 새로 실행하고 그 주소를 반환한다.
    // 백그라운드에서 애플리케이션을 구동한다.
    pub async fn spawn_app() -> Result<Self, Z2PAError> {
        Self::set_tracing();
        let mut test_app = Self::init().await?;

        // 구성을 무작위해서 테스트 격리를 보장한다.
        let _ = tokio::spawn(test_app.build_test_server().await?.run());

        Ok(test_app)
    }

    // 테스트 로그 설정을 한다.
    fn set_tracing() {
        // 한번만 실행된다.
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            // 트레이트 객체를 사용해서 타입 문제를 해결했다.
            let tracing_subscriber: Box<dyn Subscriber + Send + Sync> = std::env::var("TEST_LOG")
                .map_or(
                    Box::new(get_tracing_subscriber(LevelFilter::ERROR, std::io::sink)),
                    |_| Box::new(get_tracing_subscriber(LevelFilter::TRACE, std::io::stdout)),
                );
            init_tracing_subscriber(tracing_subscriber);
        });
    }

    // `TestApp`을 초기화한다.
    async fn init() -> Result<TestApp, config::ConfigError> {
        // 설정 파일을 읽는다.
        let mut settings = Settings::get_settings()?;
        // email_mock_server를 생성하고, `settings.email_client.uri`를 변경한다.
        let email_mock_server = MockServer::start().await;
        settings.email_client.base_url = email_mock_server.uri();

        let test_app = TestApp {
            settings,
            email_mock_server,
        };

        Ok(test_app)
    }

    // 테스트 서버를 만든다.
    async fn build_test_server(&mut self) -> Result<Server, Z2PAError> {
        let tcp_listener = self.get_test_tcp_listener().await?;
        let pool = self
            .get_test_db_pool()
            .await
            .map_err(Z2PAError::DatabaseError)?;
        // 새로운 이메일 클라이언트를 만든다.
        let email_client = self.settings.email_client.get_email_client()?;
        let base_url = self.settings.application.base_url.clone();

        // 새로운 클라이언트를 `Server`에 전달한다.
        let server = Server::new(tcp_listener, pool, email_client, base_url);
        // 서버를 백그라운드로 구동한다.
        // tokio::spawn은 생성된 퓨처에 대한 핸들을 반환한다.
        // 하지만 여기에서는 사용하지 않으므로 let을 바인딩하지 않는다.
        Ok(server)
    }

    // 테스트 `TcpListener`를 생성한다.
    // `TcpListener`에 맞춰서 `TestApp`의 주소와 관련한 설정을 한다.
    // 무작위 포트로 `TestApp`을 설정한다.
    async fn get_test_tcp_listener(&mut self) -> Result<TcpListener, std::io::Error> {
        self.settings.application.port = 0;
        let tcp_listener = self.settings.application.get_listener().await?;
        // OS가 할당한 포트 번호를 추출한다.
        // 임의의 포트가 할당되므로 설정을 변경한다.
        self.settings.application.port = tcp_listener.local_addr()?.port();
        // url에 포트를 추가한다.
        self.settings.application.base_url +=
            &format!("http://{}", tcp_listener.local_addr()?.to_string());

        Ok(tcp_listener)
    }

    // 테스트 `DefaultDBPool`을 생성한다.
    // 데이터 마이그레이션을 수행한다.
    async fn get_test_db_pool(&mut self) -> Result<DefaultDBPool, Z2PADBError> {
        // 임의의 DB 이름을 생성한다.
        self.settings.database.database_name = Uuid::new_v4().to_string();
        // 데이터베이스를 생성한다.
        let pool = DefaultDBPool::connect_without_db(&self.settings.database).await?;
        pool.create_db(&self.settings.database).await?;
        // 데이터베이스를 마이그레이션 한다.
        let pool = self.settings.database.get_pool::<DefaultDBPool>().await?;
        pool.migrate().await?;

        Ok(pool)
    }

    // `MockServer`를 생성하도록 코드를 변경
    pub async fn get_mock_server(&mut self) -> MockServer {
        let mock_server = MockServer::start().await;
        self.settings.email_client.base_url = mock_server.uri();
        mock_server
    }

    pub async fn post_subscriptions(
        &self,
        body: &'static str,
    ) -> Result<reqwest::Response, anyhow::Error> {
        reqwest::Client::new()
            .post(self.subscriptions_uri()?)
            .header(
                http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
            .context("Failed to execute request.")
    }

    // 이메일 API에 대한 요청에 포함된 확인 링크를 추출한다.
    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> Result<ConfirmationLinks, anyhow::Error> {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body)?;

        // 요청 필드의 하나에서 링크를 추출한다.
        let get_link = |s: &str| -> Result<_, anyhow::Error> {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect::<Vec<_>>();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let confirmation_link = Url::parse(&raw_link)?;
            // 웹에 대해 무작위 API를 호출하지 않는 것을 확인한다.
            assert_eq!(
                confirmation_link
                    .host_str()
                    .context("Not Found: host str")?,
                "127.0.0.1"
            );
            Ok(confirmation_link)
        };

        let html = get_link(&body["HtmlBody"].as_str().context("Not Found: HtmlBody")?)?;
        let plain_text = get_link(&body["TextBody"].as_str().context("Not Found: TextBody")?)?;

        Ok(ConfirmationLinks { html, plain_text })
    }

    pub async fn post_newsletters(
        &self,
        body: serde_json::Value,
    ) -> Result<reqwest::Response, anyhow::Error> {
        reqwest::Client::new()
            .post(self.newsletters_uri()?)
            // 무작위 크리덴셜
            // `reqwest`가 인코딩/포매팅 업무를 처리한다.
            .basic_auth(Uuid::new_v4(), Some(Uuid::new_v4()))
            .json(&body)
            .send()
            .await
            .context("Failed to execute request.")
    }

    pub fn uri(&self) -> Result<Url, url::ParseError> {
        Url::parse(&format!(
            "http://{}/",
            self.settings.application.get_address()
        ))
    }

    // /subscriptions의 주소를 얻는다.
    pub fn subscriptions_uri(&self) -> Result<Url, url::ParseError> {
        self.uri()?.join("subscriptions")
    }

    pub fn subscriptions_confirm_uri(&self) -> Result<Url, url::ParseError> {
        self.uri()?.join("subscriptions/confirm")
    }

    pub fn newsletters_uri(&self) -> Result<Url, url::ParseError> {
        self.uri()?.join("newsletters")
    }
}
