use std::sync::LazyLock;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing::Subscriber;
use zero2prod_axum::{
    configuration::{DatabaseSettings, get_configuration},
    database::ZPgPool,
    email_client::EmailClient,
    startup::run,
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

/// 테스트 정보를 저장한다.
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
pub async fn spawn_app() -> Result<TestApp, anyhow::Error> {
    LazyLock::force(&TRACING);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = get_configuration()?;

    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let z_pgpool = configure_database(&configuration.database).await?;

    let sender_email = configuration
        .email_client
        .sender()
        .map_err(|e| anyhow::anyhow!(e))?;
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    )?;

    let server = run(listener, z_pgpool.clone(), email_client);
    let _ = tokio::spawn(async move {
        server.await.expect("Failed to start server.");
    });

    Ok(TestApp { address, z_pgpool })
}

/// 테스트용 데이터베이스를 생성하고 마이그레이션한다.
async fn configure_database(config: &DatabaseSettings) -> Result<ZPgPool, sqlx::Error> {
    // 데이터베이스 생성
    let mut connection = PgConnection::connect_with(&config.without_db()).await?;
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await?;

    // 데이터베이스 마이그레이션
    let z_pgpool: ZPgPool = PgPool::connect_with(config.with_db()).await?.into();
    sqlx::migrate!("./migrations")
        .run(z_pgpool.as_ref())
        .await?;

    Ok(z_pgpool)
}
