use std::sync::LazyLock;

use pretty_assertions::assert_eq;
use reqwest::{StatusCode, header};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing::Subscriber;
use zero2prod_axum::{
    configuration::{DatabaseSettings, get_configuration},
    database::{GetZPgPool, ZPgPool},
    telemetry::{get_subscriber, init_subscriber},
};

// `LazyLock`을 사용해서 한번만 실행된다.
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    let subscriber: Box<dyn Subscriber + Send + Sync> = if std::env::var("TEST_LOG").is_ok() {
        Box::new(get_subscriber(
            subscriber_name,
            default_filter_level.into(),
            std::io::stdout,
        ))
    } else {
        Box::new(get_subscriber(
            subscriber_name,
            default_filter_level.into(),
            std::io::sink,
        ))
    };
    init_subscriber(subscriber);
});

/// 애플리케이션 정보를 저장한다.
pub struct TestApp {
    /// 인스턴스 주소
    pub address: String,
    /// 커넥션 풀
    pub z_pgpool: ZPgPool,
}

/// 백그라운드에서 애플리케이션을 구동한다.  
///
/// 반환  
///     `TestApp`
async fn spawn_app() -> Result<TestApp, anyhow::Error> {
    LazyLock::force(&TRACING);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = get_configuration()?;
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();

    let z_pgpool = configure_database(&configuration.database).await?;

    let server = zero2prod_axum::startup::run(listener, z_pgpool.clone());
    let _ = tokio::spawn(async move {
        server.await.expect("Failed to start server.");
    });

    Ok(TestApp { address, z_pgpool })
}

/// 테스트용 데이터베이스를 생성하고 마이그레이션한다.
pub async fn configure_database(config: &DatabaseSettings) -> Result<ZPgPool, sqlx::Error> {
    // 데이터베이스 생성
    let mut connection = PgConnection::connect_with(&config.without_db()).await?;
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await?;

    // 데이터베이스 마이그레이션
    let z_pgpool = PgPool::connect_with(config.with_db()).await?.get_zpg_pool();
    sqlx::migrate!("./migrations")
        .run(z_pgpool.as_ref())
        .await?;

    Ok(z_pgpool)
}

/// 멀티스레드 런타임을 사용하도록 설정해야 테스트를 통과한다.  
/// [이 페이지](https://docs.rs/tokio/latest/tokio/attr.test.html)를 참고했다.
#[tokio::test(flavor = "multi_thread")]
async fn health_check_works() -> Result<(), anyhow::Error> {
    // 준비
    let address = spawn_app().await?.address;
    let client = reqwest::Client::new();

    // 실행
    let response = client.get(format!("{address}/health_check")).send().await?;

    // 확인
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), anyhow::Error> {
    // 준비
    let app = spawn_app().await?;
    let client = reqwest::Client::new();

    // 실행
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    // 확인
    assert_eq!(StatusCode::OK, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(app.z_pgpool.pg_pool.as_ref())
        .await?;
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_400_when_data_is_missing() -> Result<(), anyhow::Error> {
    // 준비
    let app_address = spawn_app().await?.address;
    let url = format!("{app_address}/subscriptions");
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 실행
        let response = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await?;

        // 확인
        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}.",
        );
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<(), anyhow::Error>
{
    // 준비
    let app = spawn_app().await?;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid_email"),
    ];

    for (body, description) in test_cases {
        // 실행
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        // 확인
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not return a 200 OK when the payload was {description}",
        );
    }

    Ok(())
}
