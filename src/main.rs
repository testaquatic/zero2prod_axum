use tracing::level_filters::LevelFilter;
use zero2prod_axum::{
    database::{basic::Zero2ProdAxumDatabase, postgres::postgrespool::PostgresPool},
    settings::Settings,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let tracing_subscriber = get_tracing_subscriber(LevelFilter::INFO, std::io::stdout);
    init_tracing_subscriber(tracing_subscriber);
    // 구성을 읽을 수 없으면 패닉에 빠진다.
    let settings = Settings::get_settings().expect("Failed to read configuration.");
    // 하드 코딩했던 `8000`을 제거한다.
    // 해당 값은 세팅에서 얻는다.
    let tcp_listener = settings
        .application
        .get_listener()
        .await
        .expect("Failed to get a TCP listener.");
    let pool = PostgresPool::connect(&settings.database).expect("Failed to connect to Postgres.");

    tracing::info!("Starting Server");
    zero2prod_axum::startup::run(tcp_listener, pool).await
}
