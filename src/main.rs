use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;
use zero2prod_axum::{
    configuration::get_configuration,
    database::GetZPgPool,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "zero2prod_axum".into(),
        EnvFilter::from("info"),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let z_pgpool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db())
        .get_zpg_pool();
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener, z_pgpool).await
}
