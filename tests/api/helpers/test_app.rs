use std::sync::Once;

use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, Params, PasswordHasher};
use tokio::net::TcpListener;
use tracing::{level_filters::LevelFilter, Subscriber};
use url::Url;
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod_axum::{
    settings::{DefaultDBPool, Settings},
    startup::{AppState, PgSessionStorage, Server},
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

use super::DefaultDBPoolTestExt;

pub struct TestApp {
    pub settings: Settings,
    pub email_mock_server: MockServer,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

/// 이메일 API에 대한 요청에 포함된 확인 링크
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> TestUser {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(
        &self,
        pool: &DefaultDBPool,
    ) -> Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // 정확한 Argon2 파라미터에 관해서는 신경쓰지 않는다.
        // 이들은 테스트 목적이기 때문이다.

        // 기본 비밀번호의 파라미터를 매칭한다.
        let password_hash = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(19456, 2, 1, None).context("Failed to create prams.")?,
        )
        .hash_password(self.password.as_bytes(), &salt)?
        .to_string();
        pool.store_test_user(&self.user_id, &self.username, &password_hash)
            .await
            .context("Failed to store user credentials.")
    }
}

impl TestApp {
    /// 애플리케이션 인스턴스를 새로 실행하고 그 주소를 반환한다.
    // 백그라운드에서 애플리케이션을 구동한다.
    pub async fn spawn_app() -> Result<Self, anyhow::Error> {
        Self::set_tracing();
        let mut test_app = Self::init().await?;

        // 구성을 무작위해서 테스트 격리를 보장한다.
        let _ = tokio::spawn(test_app.create_test_server().await?.run());

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
    async fn init() -> Result<TestApp, anyhow::Error> {
        // 설정 파일을 읽는다.
        let mut settings = Settings::get_settings()?;
        // email_mock_server를 생성하고, `settings.email_client.uri`를 변경한다.
        let email_mock_server = MockServer::start().await;
        settings.email_client.base_url = email_mock_server.uri();
        let api_client = TestApp::get_reqwest_client()?;

        // `TestUser`를 생성한다.
        let test_user = TestUser::generate();

        let test_app = TestApp {
            settings,
            email_mock_server,
            test_user,
            api_client,
        };

        Ok(test_app)
    }

    // 테스트 서버를 만든다.
    async fn create_test_server(&mut self) -> Result<Server, anyhow::Error> {
        let tcp_listener = self.create_test_tcp_listener().await?;
        let pool = self.create_test_db_pool().await?;
        // 새로운 이메일 클라이언트를 만든다.
        let email_client = self.settings.email_client.get_email_client()?;

        let app_state = AppState::new(
            &self.settings.application.hmac_secret,
            pool.clone(),
            email_client,
            &self.settings.application.base_url,
        )?;

        let session_strage =
            PgSessionStorage::init(pool, self.settings.application.hmac_secret.clone()).await?;

        // 새로운 클라이언트를 `Server`에 전달한다.
        let server = Server::new(tcp_listener, app_state, session_strage);
        // 서버를 백그라운드로 구동한다.
        // tokio::spawn은 생성된 퓨처에 대한 핸들을 반환한다.
        // 하지만 여기에서는 사용하지 않으므로 let을 바인딩하지 않는다.
        Ok(server)
    }

    // 테스트 `TcpListener`를 생성한다.
    // `TcpListener`에 맞춰서 `TestApp`의 주소와 관련한 설정을 한다.
    // 무작위 포트로 `TestApp`을 설정한다.
    async fn create_test_tcp_listener(&mut self) -> Result<TcpListener, std::io::Error> {
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
    // 데이터 마이그레이션과 테스트에 필요한 데이터를 DB에 저장한다.
    async fn create_test_db_pool(&mut self) -> Result<DefaultDBPool, anyhow::Error> {
        // 임의의 DB 이름을 생성한다.
        self.settings.database.database_name = Uuid::new_v4().to_string();
        // 데이터베이스를 생성한다.
        DefaultDBPool::create_db(&self.settings.database).await?;
        // 데이터베이스를 마이그레이션 한다.
        let pool = self.settings.database.get_pool::<DefaultDBPool>().await?;
        pool.migrate().await?;
        // `TestUser`를 DB에 저장한다.
        self.test_user.store(&pool).await?;

        Ok(pool)
    }

    pub async fn post_subscriptions(
        &self,
        body: &'static str,
    ) -> Result<reqwest::Response, anyhow::Error> {
        self.api_client
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

    pub fn get_reqwest_client() -> Result<reqwest::Client, reqwest::Error> {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            .build()
    }

    pub async fn post_newsletters(
        &self,
        body: serde_json::Value,
    ) -> Result<reqwest::Response, anyhow::Error> {
        self.api_client
            .post(self.newsletters_uri()?)
            // 더 이상 무작위로 생성하지 않는다.
            // `reqwest`가 인코딩/포매팅 업무를 처리한다.
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .context("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> Result<reqwest::Response, anyhow::Error>
    where
        Body: serde::Serialize,
    {
        let response = self
            .api_client
            .post(self.login_uri()?)
            // 이 `reqwest` 메서드는 바디가 url 인코딩되어 있으며 `Content-Type`헤드가 그에 따라 설정되어 있음을 보장한다.
            .form(body)
            .send()
            .await?;
        Ok(response)
    }

    // 테스트 케이스는 HTML 페이지만 확인한다.
    // 따라서 기반 reqwest::Response는 노출하지 않는다.
    pub async fn get_login_html(&self) -> Result<String, anyhow::Error> {
        let result = self
            .api_client
            .get(self.login_uri()?)
            .send()
            .await
            .context("Failed to execute request.")?
            .text()
            .await?;

        Ok(result)
    }

    pub async fn get_admin_dashboard(&self) -> Result<String, anyhow::Error> {
        let result = self
            .api_client
            .get(self.uri()?.join("admin/dashboard")?)
            .send()
            .await
            .context("Failed to execute request.")?
            .text()
            .await?;
        
        Ok(result)
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

    pub fn login_uri(&self) -> Result<Url, url::ParseError> {
        self.uri()?.join("login")
    }
}
