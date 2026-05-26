use std::sync::LazyLock;

use sqlx::{Connection, QueryBuilder, postgres};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod_axum::{
    configuration::{self, Settings},
    startup::Application,
    telemetry,
};

use crate::helpers::{test_app::TestApp, test_user::TestUser};

/// tracing
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber(default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber)
    } else {
        let subscriber = telemetry::get_subscriber(default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber)
    }
});

/// 앱을 백그라운드에서 실행한다.
pub async fn spawn_app() -> TestApp {
    // tracing
    // 한번만 불러온다.
    let _ = LazyLock::force(&TRACING);

    // 설정을 읽는다
    let mut configuration =
        configuration::get_configuration().expect("failed to read configuration");

    // 테스트용의 email server
    let email_server = MockServer::start().await;
    configuration.email_client.base_url = email_server.uri();

    // 데이터베이스 마이그레이션
    configuration.database.database_name = Uuid::new_v4().to_string();
    migrate_test_database(&configuration).await;

    // `EmailClient`의 타입아웃을 200ms로 설정한다.
    configuration.email_client.timeout_milliseconds = 200;

    // 테스트 유저를 넣는다
    let test_user = TestUser::generate();
    test_user
        .store(&zero2prod_axum::startup::get_connection_pool(
            &configuration,
        ))
        .await;

    // `TcpListener` 생성
    // port를 0으로 설정
    configuration.application.port = 0;
    let application = Application::build(configuration.clone())
        .await
        .expect("failed to build application");
    // listener의 포트를 반영한다.
    let application_port = application.port().await.expect("failed to get port");
    configuration.application.port = application_port;
    configuration.application.base_url = format!("http://localhost:{}", application_port);

    // 테스트 서버 실행
    let api_server_handle = tokio::spawn(application.run_until_stopped());

    TestApp {
        configuration,
        email_server,
        test_user,
        _api_server_handle: api_server_handle,
    }
}

/// 테스트 DB를 생성한다.
async fn migrate_test_database(config: &Settings) {
    let mut connection = postgres::PgConnection::connect_with(&config.database.without_db())
        .await
        .expect("failed to connect to db");

    QueryBuilder::new(format!(
        r#"CREATE DATABASE "{}";"#,
        config.database.database_name
    ))
    .build()
    .execute(&mut connection)
    .await
    .expect("failed to create db");

    let pg_pool = zero2prod_axum::startup::get_connection_pool(&config);

    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("failed to migrate the database");
}
