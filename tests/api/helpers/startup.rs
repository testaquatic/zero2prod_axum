use std::sync::LazyLock;

use sqlx::{Connection, QueryBuilder, postgres};
use uuid::Uuid;
use zero2prod_axum::{
    configuration::{self, Settings},
    startup::{Application, get_connection_pool},
    telemetry,
};

use crate::helpers::testapp::TestApp;

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
    let _ = LazyLock::force(&TRACING);

    let mut configuration =
        configuration::get_configuration().expect("failed to read configuration");

    // 데이터베이스 마이그레이션
    configuration.database.database_name = Uuid::new_v4().to_string();
    migrate_test_database(&configuration).await;

    // `EmailClient`의 타입아웃을 200ms로 설정한다.
    configuration.email_client.timeout_milliseconds = 200;

    // 포트를 0으로 일단 설정한다.
    configuration.application.port = 0;

    let application = Application::build(configuration.clone())
        .await
        .expect("failed to build application");

    // listener의 포트를 반영한다.
    configuration.application.port = application.port().await.expect("failed to get port");

    let _server_handle = tokio::spawn(application.run_until_stopped());

    TestApp { configuration }
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

    let pg_pool = get_connection_pool(&config);

    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("failed to migrate the database");
}
