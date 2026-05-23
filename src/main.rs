use zero2prod_axum::{startup, telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("info", std::io::stdout);
    telemetry::init_subscriber(subscriber);

    startup::run().await
}
