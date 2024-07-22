use tracing::level_filters::LevelFilter;
use zero2prod_axum::{
    settings::Settings,
    startup::Server,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let tracing_subscriber = get_tracing_subscriber(LevelFilter::INFO, std::io::stdout);
    init_tracing_subscriber(tracing_subscriber);
    // 구성을 읽을 수 없으면 패닉에 빠진다.
    let settings = Settings::get_settings().expect("Failed to read configuration.");
    let tcp_listener = settings
        .application
        .get_listener()
        .await
        .expect("Failed to get a TCP listener.");
    let pool = settings
        .database
        .get_pool()
        .await
        .expect("Failed to connect to Postgres.");
    // `settings`를 사용해서 `EmailClient`를 만든다.
    let email_client = settings
        .email_client
        .get_email_client()
        .expect("Failed to get EmailClient.");

    tracing::info!(name: "server", status = "Starting server", addr = %tcp_listener.local_addr().unwrap().to_string());
    // `run`, `email_client`를 위한 새로운 인자
    Server::new(tcp_listener, pool, email_client).run().await
}
