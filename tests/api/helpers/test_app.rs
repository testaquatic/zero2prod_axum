use std::sync::Once;

use tokio::net::TcpListener;
use tracing::{level_filters::LevelFilter, Subscriber};
use url::Url;
use uuid::Uuid;
use wiremock::{Mock, MockServer};
use zero2prod_axum::{
    error::Zero2ProdAxumError,
    settings::{DefaultDBPool, Settings},
    startup::Server,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

use super::DefaultDBPoolTestExt;

pub struct TestApp {
    pub settings: Settings,
    // settings만으로 테스트를 수행하고 싶지만 테스트 이메일 서버의 주소를 파악하려면 인스턴스가 필요하다.
    // `&mut`을 사용하면 되겠지만 나중에 혼란을 줄 것 같다.
    pub test_email_server: TestEmailServer,
}

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

    pub async fn received_requests(&self) -> Option<Vec<wiremock::Request>> {
        self.mock_server.received_requests().await
    }
}

impl TestApp {
    /// 애플리케이션 인스턴스를 새로 실행하고 그 주소를 반환한다.
    // 백그라운드에서 애플리케이션을 구동한다.
    pub async fn spawn_app() -> Result<Self, Zero2ProdAxumError> {
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

    async fn init() -> Result<TestApp, config::ConfigError> {
        let mut settings = Settings::get_settings()?;
        // MockServer를 구동해서 Postmark의 API를 대신한다.
        let test_email_server = TestEmailServer::new().await;

        // email_server를 이메일 API로 사용한다.
        settings.email_client.base_url = test_email_server.uri();

        let test_app = TestApp {
            settings,
            test_email_server,
        };

        Ok(test_app)
    }

    // 테스트 서버를 만든다.
    async fn build_test_server(&mut self) -> Result<Server, Zero2ProdAxumError> {
        let tcp_listener = self.get_test_tcp_listener().await?;
        let pool = self.get_test_db_pool().await?;
        // 새로운 이메일 클라이언트를 만든다.
        let email_client = self.settings.email_client.get_email_client()?;

        // 새로운 클라이언트를 `Server`에 전달한다.
        let server = Server::new(tcp_listener, pool, email_client);
        // 서버를 백그라운드로 구동한다.
        // tokio::spawn은 생성된 퓨처에 대한 핸들을 반환한다.
        // 하지만 여기에서는 사용하지 않으므로 let을 바인딩하지 않는다.
        Ok(server)
    }

    // 테스트 `TcpListener`를 생성한다.
    // 무작위 포트로 `TestApp`을 설정한다.
    async fn get_test_tcp_listener(&mut self) -> Result<TcpListener, std::io::Error> {
        self.settings.application.port = 0;
        let tcp_listener = self.settings.application.get_listener().await?;
        // OS가 할당한 포트 번호를 추출한다.
        // 임의의 포트가 할당되므로 설정을 변경한다.
        self.settings.application.port = tcp_listener.local_addr()?.port();

        Ok(tcp_listener)
    }

    // 테스트 `DefaultDBPool`을 생성한다.
    // 데이터 마이그레이션을 수행한다.
    async fn get_test_db_pool(&mut self) -> Result<DefaultDBPool, sqlx::Error> {
        // 임의의 DB 이름을 생성한다.
        self.settings.database.database_name = Uuid::new_v4().to_string();
        // 데이터베이스를 생성한다.
        let pool = DefaultDBPool::connect_without_db(&self.settings.database).await?;
        pool.create_db(&self.settings.database).await?;
        // 데이터베이스를 마이그레이션 한다.
        let pool = self.settings.database.get_pool().await?;
        pool.migrate().await?;

        Ok(pool)
    }

    pub fn get_uri(&self) -> Result<Url, url::ParseError> {
        Url::parse(&format!(
            "http://{}/",
            self.settings.application.get_address()
        ))
    }

    // /subscriptions의 주소를 얻는다.
    pub fn get_subscriptions_uri(&self) -> Result<Url, url::ParseError> {
        self.get_uri()?.join("subscriptions")
    }

    pub async fn post_subscriptions(
        &self,
        body: &'static str,
    ) -> Result<reqwest::Response, Zero2ProdAxumError> {
        reqwest::Client::new()
            .post(self.get_subscriptions_uri()?)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
            .map_err(Zero2ProdAxumError::ReqwestError)
    }
}
