use tracing_subscriber::EnvFilter;
use zero2prod_axum::{
    configuration::get_configuration,
    startup::Application,
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
    // 서버 인스턴스를 얻는다.
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
