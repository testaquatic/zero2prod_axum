use std::sync::Once;

use sqlx::{postgres::PgQueryResult, Database, Executor, FromRow};
use tokio::net::TcpListener;
use tracing::{level_filters::LevelFilter, Subscriber};
use uuid::Uuid;
use zero2prod_axum::{
    database::{
        basic::Zero2ProdAxumDatabase,
        postgres::postgrespool::{DatabaseSettingsExt, PostgresPool},
    },
    settings::{DatabaseSettings, DefaultDBPool, Settings},
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

trait DefaultDBPoolTestExt: Zero2ProdAxumDatabase {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, sqlx::Error>;

    async fn create_db(
        &self,
        database_settings: &DatabaseSettings,
    ) -> Result<<Self::DB as Database>::QueryResult, sqlx::Error>;

    async fn fetch_one(&self, query: &str) -> Result<<Self::DB as Database>::Row, sqlx::Error>;

    async fn execute(
        &self,
        query: &str,
    ) -> Result<<Self::DB as Database>::QueryResult, sqlx::Error>;

    async fn migrate(&self) -> Result<(), sqlx::Error>;
}

impl DefaultDBPoolTestExt for PostgresPool {
    async fn connect_without_db(database_settings: &DatabaseSettings) -> Result<Self, sqlx::Error> {
        let connect_options = database_settings.connect_options_without_db();
        let pool = sqlx::PgPool::connect_with(connect_options).await?;
        Ok(Self::new(pool))
    }

    async fn create_db(
        &self,
        database_settings: &DatabaseSettings,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let pool = Self::connect_without_db(database_settings).await?;

        pool.execute(format!(r#"CREATE DATABASE "{}""#, database_settings.database_name).as_str())
            .await
    }

    async fn fetch_one(
        &self,
        query: &str,
    ) -> Result<<<Self as Zero2ProdAxumDatabase>::DB as Database>::Row, sqlx::Error> {
        self.as_ref().fetch_one(query).await
    }

    async fn execute(&self, query: &str) -> Result<PgQueryResult, sqlx::Error> {
        self.as_ref().execute(query).await
    }

    async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(self.as_ref())
            .await
            .expect("Failed to migrate database.");
        Ok(())
    }
}

pub struct TestApp {
    pub settings: Settings,
}

impl TestApp {
    /// 애플리케이션 인스턴스를 새로 실행하고 그 주소를 반환한다.
    // 백그라운드에서 애플리케이션을 구동한다.
    async fn spawn_app() -> Self {
        Self::set_tracing();
        let settings = Settings::get_settings().expect("Failed to read settings");
        let mut test_app = TestApp { settings };
        let tcp_listener = test_app.get_tcp_listener().await;
        let pool = test_app.get_test_db_pool().await;

        let server = zero2prod_axum::startup::run(tcp_listener, pool);
        // 서버를 백그라운드로 구동한다.
        // tokio::spawn은 생성된 퓨처에 대한 핸들을 반환한다.
        // 하지만 여기에서는 사용하지 않으므로 let을 바인딩하지 않는다.
        let _ = tokio::spawn(server);

        test_app
    }

    // /subscriptions의 주소를 얻는다.
    fn subscriptions_uri(&self) -> String {
        format!(
            "http://{}/subscriptions",
            self.settings.application.get_address()
        )
    }

    // TCP 설정
    async fn get_tcp_listener(&mut self) -> TcpListener {
        let tcp_listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind address to listener.");
        // OS가 할당한 포트 번호를 추출한다.
        // 임의의 포트가 할당되므로 설정을 변경한다.
        self.settings.application.port = tcp_listener.local_addr().unwrap().port();

        tcp_listener
    }

    // DB 설정
    async fn get_test_db_pool(&mut self) -> DefaultDBPool {
        // 임의의 DB 이름을 생성한다.
        self.settings.database.database_name = Uuid::new_v4().to_string();
        // 데이터베이스를 생성한다.
        let pool = DefaultDBPool::connect_without_db(&self.settings.database)
            .await
            .expect("Failed to connect to database.");
        pool.create_db(&self.settings.database)
            .await
            .expect("Failed to create database.");
        // 데이터베이스를 마이그레이션 한다.
        let pool =
            DefaultDBPool::connect(&self.settings.database).expect("Failed to connect Database.");
        pool.migrate().await.expect("Failed to migrate database.");

        pool
    }

    fn set_tracing() {
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
}

// `tokio::test`는 테스팅에 있어서 `tokio::main`과 동등하다.
// `#[test]` 속성을 지정하는 수고를 덜 수 있다.
//
// `cargo expand --test health_check`를 사용해서 코드가 무엇을 생성하는지 확인할 수 있다.
#[tokio::test]
async fn health_check_works() {
    // 준비
    let test_app = TestApp::spawn_app().await;
    // `reqwest`를 사용해서 애플리케이션에 대한 HTTP 요청을 수행한다.
    let client = reqwest::Client::new();

    //실행
    let response = client
        .get(format!(
            "http://{}/health_check",
            test_app.settings.application.get_address()
        ))
        .send()
        .await
        .expect("Failed to execute request.");
    // 확인
    // 응답 상태 코드가 OK인지 확인한다.
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    // 응답 본문의 길이가 0인지 확인한다.
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // 테스트 데이터
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 준비
    let test_app = TestApp::spawn_app().await;
    let pool = DefaultDBPool::connect(&test_app.settings.database)
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    // 실행
    let response = client
        .post(test_app.subscriptions_uri())
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    let row = pool
        .fetch_one("SELECT email, name FROM subscriptions;")
        .await
        .expect("Failed to fetch saved subscriptions.");

    // 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    #[derive(sqlx::FromRow)]
    struct Saved {
        email: String,
        name: String,
    }
    let saved = Saved::from_row(&row).expect("Failed to get data from Database.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin")
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    // 테스트 데이터
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    // 준비
    let test_app = TestApp::spawn_app().await;
    let client = reqwest::Client::new();

    for (invalid_body, error_messages) in test_cases {
        // 실행
        let response = client
            .post(test_app.subscriptions_uri())
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // 확인
        assert_eq!(
            response.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            // 테스트 실패시 출력할 커스터마이즈된 추가 오류 메세지
            "The API did not fail with 400 Bad Request when the payload was {},",
            error_messages,
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // 테스트 데이터
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitelyNotAnEmail", "invalid email"),
    ];

    // 준비
    let app = TestApp::spawn_app().await;
    let client = reqwest::Client::new();

    for (body, description) in test_cases {
        // 실행
        let response = client
            .post(app.subscriptions_uri())
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        // 확인
        assert_eq!(
            // 더이상 200 OK가 아니다.
            reqwest::StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}
