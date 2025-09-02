use std::sync::{Arc, LazyLock};

use argon2::{Algorithm, Argon2, Params, Version};
use migration::MigratorTrait;
use password_hash::{PasswordHasher, Salt};
use rand::distr::SampleString;
use reqwest::{StatusCode, header};
use sea_orm::{ActiveValue, ConnectionTrait, DatabaseConnection, EntityTrait, sqlx::PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod_axum::{
    configuration::{DatabaseSettings, get_configuration},
    cookie::CookieFeeder,
    startup::{Applicaton, get_connection_pool},
    telemetry::{get_subscriber, init_subscriber},
};

/// `LazyLock`을 사용해서 한번만 초기화 되는 것을 보장한다.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info,axum::rejection=trace,tower_http=debug".to_string();

    std::env::var("TEST_LOG")
        .map(|_| {
            let subscriber = get_subscriber(default_filter_level.clone(), std::io::stdout);
            init_subscriber(subscriber);
        })
        // `unwrap_or`는 부지런하기 때문에 오류가 발생한다.
        .unwrap_or_else(|_| {
            let subscriber = get_subscriber(default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        });
});

/// 테스트용 어플리케이션 구조체
pub struct TestApp {
    /// 서버의 주소
    pub address: String,
    /// 데이터베이스 연결 풀
    pub db_pool: DatabaseConnection,
    /// 이메일 서버를 모사한다.
    pub email_server: MockServer,
    /// 이메일 서버의 포트이다.
    pub port: u16,
    /// 테스트용 사용자 정보이다.
    pub test_user: TestUser,
    /// 테스트용 reqwest 인스턴스이다.
    pub api_client: reqwest::Client,
    /// 쿠키를 저장한다.
    pub cookie_store: Arc<reqwest_cookie_store::CookieStoreMutex>,
}

/// 이메일 API에 대한 요청에 포함된 확인 링크
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

/// 테스트용 사용자 정보를 나타내는 구조체
pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestApp {
    /// /subscriptions 엔드포인트에 POST 요청을 보내는 헬퍼 메서드이다.
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/subscriptions", &self.address))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// 이메일 API에 대한 요청에 포함된 확인 링크를 추출한다.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body = email_request.body_json::<serde_json::Value>().unwrap();

        let html = self.get_links(&body["HtmlBody"].as_str().unwrap());
        let plain_text = self.get_links(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    /// 문자열에 있는 링크들을 모은 벡터를 얻는다.
    fn get_links(&self, s: &str) -> reqwest::Url {
        let links = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect::<Vec<_>>();

        assert_eq!(links.len(), 1);
        let raw_link = links[0].as_str().to_string();
        let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

        // 웹에서 무작위 API를 호출하지 않는 것을 확인한다.
        assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

        confirmation_link.set_port(Some(self.port)).unwrap();

        confirmation_link
    }

    /// /newsletters 엔드포인트에 POST 요청을 보내는 헬퍼 메서드이다.
    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// /login 엔드포인트에 POST 요청을 보내는 헬퍼 메서드이다.
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// _flash 쿠기의 내용을 확인하다.
    /// 반환 값은 (message, hmac)이다.
    pub async fn get_manage_cookies(&self, path: &str) -> Option<CookieFeeder> {
        // https://docs.rs/reqwest_cookie_store/0.9.0/reqwest_cookie_store/index.html 이 문서를 참고로 했다.
        let cookie_header = self.cookie_store.lock().unwrap();

        let hmac = cookie_header
            .get("127.0.0.1", path, CookieFeeder::HMAC)
            .map(|cookie| cookie.value().to_string());
        let message = cookie_header
            .get("127.0.0.1", path, CookieFeeder::MESSAGE)
            .map(|cookie| cookie.value().to_string());
        let username = cookie_header
            .get("127.0.0.1", path, CookieFeeder::USERNAME)
            .map(|cookie| cookie.value().to_string());

        Some(CookieFeeder::new(message, username, hmac, None))
    }

    pub async fn post_check_cookie(&self, body: serde_json::Value) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/check/hmac", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }
}

impl TestUser {
    /// 테스트용 유저 정보를 생성한다.
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    /// 데이터베이스에 테스트용 유저를 저장한다.
    async fn store(&self, pool: &DatabaseConnection) {
        // rand 0.9는 SaltString::generate를 사용할 수 없어서 좀 돌아갔다.
        let salt_string =
            rand::distr::Alphanumeric.sample_string(&mut rand::rng(), Salt::MAX_LENGTH);
        let salt = Salt::from_b64(&salt_string).unwrap();

        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19 * 1024, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), salt)
        .unwrap()
        .to_string();

        let new_user_model = entities::users::ActiveModel {
            user_id: ActiveValue::Set(self.user_id),
            username: ActiveValue::Set(self.username.to_string()),
            password_hash: ActiveValue::Set(password_hash),
        };

        entities::users::Entity::insert(new_user_model)
            .exec(pool)
            .await
            .expect("Failed to store test user.");
    }
}

/// 서버를 실행하는 헬퍼 함수
/// 서버의 주소를 반환한다.(예: http://localhost:8000)
pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    // Postmark의 API를 대신한다.
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();

        c
    };

    configure_database(&configuration.database).await;

    let application = Applicaton::from_settings(&configuration)
        .await
        .expect("Failed to build application.");
    let application_port = application.port();

    let _ = tokio::spawn(application.run());

    // 서버의 시작을 기다린다.
    // 이 부분이 없어도 오류가 발생하지 않아서 임시로 주석처리 했다.
    // tokio::time::sleep(Duration::from_millis(100)).await;
    let cookie_store = Arc::new(reqwest_cookie_store::CookieStoreMutex::new(
        reqwest_cookie_store::CookieStore::new(),
    ));

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_provider(Arc::clone(&cookie_store))
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        port: application_port,
        test_user: TestUser::generate(),
        api_client: client,
        cookie_store,
    };
    test_app.test_user.store(&test_app.db_pool).await;

    test_app
}

/// 테스트용 데이터베이스를 설정하는 헬퍼 함수
/// 데이터베이스를 생성하고, 마이그레이션을 적용한다.
async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let connection = sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(
        PgPool::connect_with(config.without_db())
            .await
            .expect("Failed to connect to Postgres."),
    );

    // 데이터 생성과 관련한 API를 찾지 못했다.
    // 날SQL을 사용하여 데이터베이스를 생성한다.
    connection
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(r#"CREATE DATABASE "{}";"#, config.database_name),
        ))
        .await
        .expect("Failed to create database.");

    let connection_pool = sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(
        PgPool::connect_with(config.with_db())
            .await
            .expect("Failed to connect to Postgres."),
    );
    // 데이터베이스를 마이그레이션 한다.
    // https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/ 문서를 참고했다.
    migration::Migrator::up(&connection_pool, None)
        .await
        .expect("Failed to migrate database.");

    connection_pool
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), location);
}
