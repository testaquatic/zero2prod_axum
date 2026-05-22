use zero2prod_axum::{
    configuration::{self, DatabasettingsExt},
    domain::Settings,
    startup,
};

/// 앱을 백그라운드에서 실행한다.
/// 반환값
/// 주소의 형식은 http://127.0.0.1:무작위포트 이다.
pub async fn spawn_app() -> Result<(tokio::task::JoinHandle<()>, Settings), std::io::Error> {
    let mut configuration =
        configuration::get_configuration().expect("failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let pool = sqlx::PgPool::connect(&connection_string)
        .await
        .expect("failed to connect to db");

    let router = startup::get_app_router(pool);
    let listner = tokio::net::TcpListener::bind("127.0.0.1:0").await?;

    let port = listner.local_addr().unwrap().port();

    configuration.application_port = port;

    let server_handle = tokio::spawn(async move {
        axum::serve(listner, router)
            .await
            .expect("failed to start server")
    });

    Ok((server_handle, configuration))
}
