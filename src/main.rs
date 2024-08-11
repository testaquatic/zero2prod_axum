use std::fmt::{Debug, Display};

use tokio::task::JoinError;
use tracing::level_filters::LevelFilter;
use zero2prod_axum::{
    issue_delivery_worker::run_worker_until_stopped,
    settings::Settings,
    telemetry::{get_tracing_subscriber, init_tracing_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let tracing_subscriber = get_tracing_subscriber(LevelFilter::INFO, std::io::stdout);
    init_tracing_subscriber(tracing_subscriber);
    // 구성을 읽을 수 없으면 패닉에 빠진다.
    let settings = Settings::get_settings()?;
    let server = tokio::spawn(settings.build_server().await?.run());
    let worker = tokio::spawn(run_worker_until_stopped(settings));

    tokio::select! {
        o = server => report_ext("API", o),
        o = worker => report_ext("Backgroud worker", o),
    };

    Ok(())
}

fn report_ext(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => tracing::info!("{} has exited", task_name),
        Ok(Err(e)) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} failed", task_name)
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} task failed to complete", task_name)
        }
    }
}
