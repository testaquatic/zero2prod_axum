use std::sync::LazyLock;

use migration::MigratorTrait;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, EntityTrait};
use secrecy::ExposeSecret;
use uuid::Uuid;
use zero2prod_axum::{
    configuration::{DatabaseSettings, get_configuration},
    entities::prelude::Subscriptions,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

/// `LazyLock`을 사용해서 한번만 초기화 되는 것을 보장한다.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();

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
}

/// 서버를 실행하는 헬퍼 함수
/// 서버의 주소를 반환한다.(예: http://localhost:8000)
async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone());
    let _ = tokio::spawn(server);
    // 서버의 시작을 기다린다.
    // 이 부분이 없어도 오류가 발생하지 않아서 임시로 주석처리 했다.
    // tokio::time::sleep(Duration::from_millis(100)).await;

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

/// 테스트용 데이터베이스를 설정하는 헬퍼 함수
/// 데이터베이스를 생성하고, 마이그레이션을 적용한다.
async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let connection = Database::connect(config.connection_string_without_db().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    // 데이터 생성과 관련한 API를 찾지 못했다.
    // 날SQL을 사용하여 데이터베이스를 생성한다.
    connection
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!(r#"CREATE DATABASE "{}";"#, config.database_name),
        ))
        .await
        .expect("Failed to create database.");

    let connection_pool = Database::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    // 데이터베이스를 마이그레이션 한다.
    // https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/ 문서를 참고했다.
    migration::Migrator::up(&connection_pool, None)
        .await
        .expect("Failed to migrate database.");

    connection_pool
}

/// /health_check에 GET요청을 보내면 200 OK를 반환해야 한다.
/// 응답에는 바디가 없다.
#[tokio::test]
async fn health_check_works() {
    // 준비
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // 실행
    let response = client
        .get(&format!("{}/health_check", app.address.as_str()))
        .send()
        .await
        .expect("Failed to send request.");

    // 확인
    // 응답 상태 코드가 200 OK인지 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    // 응답 바디가 비어 있는지 확인
    assert_eq!(response.content_length(), Some(0));
}

/// /subscriptions에 유효한 POST요청을 보내면 200 OK를 반환해야 한다.
/// 요청의 Content-Type은 application/x-www-form-urlencoded이어야 한다.
/// 요청 바디에는 name과 email 필드가 있어야 한다.
/// 데이터베이스에 구독 정보가 저장되어야 한다.
/// 구독 정보는 name과 email 필드를 포함해야 한다.
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // 준비
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // 실행
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", app.address.as_str()))
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // 확인
    // 응답 상태 코드가 200 OK인지 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let saved = Subscriptions::find()
        .one(&app.db_pool)
        .await
        .expect("Failed to fetch subscription.");
    // 데이터베이스에 저장된 구독 정보가 올바른지 확인
    assert!(
        saved.is_some(),
        "No subscription was saved to the database."
    );
    assert_eq!(
        saved.as_ref().unwrap().name,
        "le guin",
        "The saved name does not match the expected value."
    );
    assert_eq!(
        saved.as_ref().unwrap().email,
        "ursula_le_guin@gmail.com",
        "The saved email does not match the expected value."
    );
}

/// /subscriptions에 name이나 email 필드가 없는 POST요청을 보내면 400 Bad Request를 반환해야 한다.
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // 준비
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 실행
        let response = client
            .post(&format!("{}/subscriptions", app.address.as_str()))
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // 확인
        // actix-web하고 다르게 axum은 422 Unprocessable Entity를 반환한다.
        assert_eq!(
            response.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not return a 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}
