use sqlx::{Connection, QueryBuilder, postgres};
use uuid::Uuid;
use zero2prod_axum::{
    configuration::{self, DatabaseSettingsExt},
    domain::DatabaseSettings,
    startup,
};

use crate::helper::testapp::TestApp;

mod testapp;

/// 앱을 백그라운드에서 실행한다.
/// 반환값
/// 주소의 형식은 http://127.0.0.1:무작위포트 이다.
pub async fn spawn_app() -> TestApp {
    let mut configuration =
        configuration::get_configuration().expect("failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let router = startup::get_app_router(connection_pool);
    let listner = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind address");

    let port = listner.local_addr().unwrap().port();

    configuration.application_port = port;

    let server_handle = tokio::spawn(async move {
        axum::serve(listner, router)
            .await
            .expect("failed to start server")
    });

    TestApp::new(configuration, server_handle)
}

/// 테스트 DB를 생성한다.
async fn configure_database(config: &DatabaseSettings) -> sqlx::PgPool {
    let mut connection = postgres::PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("failed to connect to db");

    QueryBuilder::new(format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .build()
        .execute(&mut connection)
        .await
        .expect("failed to create db");

    let connection_pool = sqlx::PgPool::connect(&config.connection_string())
        .await
        .expect("failed to connect to db");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate the database");

    connection_pool
}
