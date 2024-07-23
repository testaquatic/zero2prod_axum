use tracing::level_filters::LevelFilter;
use zero2prod_axum::{
    settings::Settings,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let tracing_subscriber = get_tracing_subscriber(LevelFilter::INFO, std::io::stdout);
    init_tracing_subscriber(tracing_subscriber);
    // 구성을 읽을 수 없으면 패닉에 빠진다.
    Settings::get_settings()?
        .build_server()
        .await?
        .run()
        .await?;

    Ok(())
}
