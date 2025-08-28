use anyhow::Context;
use zero2prod_axum::{
    configuration::get_configuration,
    startup::Applicaton,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // tracing 관련 초기화를 한다.
    let subscriber = get_subscriber("debug,axum::rejection=trace".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().context("Failed to read configuration")?;

    let application = Applicaton::from_settings(&configuration)
        .await
        .context("Failed to build application")?;
    application.run().await.context("Failed to run server")
}
