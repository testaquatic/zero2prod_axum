use sea_orm::Database;
use secrecy::ExposeSecret;
use zero2prod_axum::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool =
        Database::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    run(listener, connection_pool).await
}
