use zero2prod_axum::{
    configuration::get_configuration,
    startup::Applicaton,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // tracing 관련 초기화를 한다.
    let subscriber = get_subscriber("debug,axum::rejection=trace".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let server = Applicaton::build(&configuration).await?;
    server.run_until_stopped().await
}
