use std::{
    fmt::{Debug, Display},
    io,
    time::Duration,
};

use axum::Router;
use moka::future::Cache;
use secrecy::{ExposeSecret, SecretSlice};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{net::TcpListener, task::JoinError};
use uuid::Uuid;

use crate::{
    app_state::{self},
    configuration::{self, Settings},
    email_client::{self},
    router::get_app_router,
    service::newsletter::NewsletterService,
};

pub fn get_connection_pool(configuration: &configuration::Settings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy_with(configuration.database.with_db())
}

/// `Settings`로부터 `EmailClient`를 생성한다.
/// 이메일의 형식이 정상적인지 확인하고, 정상적이지 않다면 Err(String)을 반환한다.
pub fn get_email_client(
    email_client_settings: &configuration::EmailClientSettings,
) -> Result<email_client::EmailClient, String> {
    // SubscriberEmail 생성
    let sender_email = email_client_settings.sender()?;
    // EmailClient 생성
    let email_client = email_client::EmailClient::new(
        email_client_settings.base_url.clone(),
        sender_email,
        email_client_settings.authorization_token.clone(),
        email_client_settings.timeout(),
    );

    Ok(email_client)
}

/// 사적이면서 공적인 나만의 시크릿
pub fn get_private_and_public_key_from_configuration(
    configuration: &Settings,
) -> Result<(SecretSlice<u8>, SecretSlice<u8>), io::Error> {
    // pem 파일 읽기
    let private_key = std::fs::read(
        configuration
            .application
            .token_secret_private_pem
            .expose_secret(),
    )?
    .into();
    let public_key = std::fs::read(
        configuration
            .application
            .token_secret_public_pem
            .expose_secret(),
    )?
    .into();

    Ok((private_key, public_key))
}

pub async fn email_worker_loop(configuration: &Settings) -> Result<(), anyhow::Error> {
    let pool = get_connection_pool(configuration);
    let email_client = get_email_client(&configuration.email_client)
        .map_err(|e| anyhow::anyhow!("Failed to get email client: {}", e))?;
    loop {
        match NewsletterService::try_execute_task(&NewsletterService, &email_client, &pool).await {
            Ok(Some(_)) => (),
            _ => {
                tokio::time::sleep(Duration::from_millis(
                    configuration.application.email_worker_interval_milliseconds,
                ))
                .await
            }
        }
    }
}

/// 서버를 실행한다
pub async fn run() -> Result<(), anyhow::Error> {
    // 설정을 읽는다
    let configuration = configuration::get_configuration().expect("failed to read configuration");

    let application = Application::build(configuration.clone())
        .await
        .expect("failed to build application");
    let email_worker = async move { email_worker_loop(&configuration).await };

    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(email_worker);

    // 서버 실행
    tokio::select! {
     result =  application_task => report_exit("API", result),
     result = worker_task => report_exit("Background worker", result),
    }

    Ok(())
}

pub fn report_exit(task_name: &str, outcome: Result<Result<(), impl Display + Debug>, JoinError>) {
    match outcome {
        Ok(Ok(_)) => tracing::info!("{} has exited", task_name),
        Ok(Err(e)) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} failed", task_name)
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} task failed to complete", task_name)
        }
    }
}

// 서버 실행에 필요한 정보를 가지고 있는 구조체
pub struct Application {
    listener: TcpListener,
    app_router: Router,
}

impl Application {
    /// `Settings로부터 서버 실행에 필요한 `Application`을 생성하는 함수`
    pub async fn build(configuration: configuration::Settings) -> Result<Self, std::io::Error> {
        // 데이터베이스 풀 생성
        let connection_pool = get_connection_pool(&configuration);

        // `EmailClient` 생성
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let timeout = configuration.email_client.timeout();

        let (private_key, public_key) =
            get_private_and_public_key_from_configuration(&configuration)?;

        let email_client = email_client::EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        // `TcpListener` 생성
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        tracing::info!("listening on {}", address);
        let listener = tokio::net::TcpListener::bind(address).await?;

        // Moka 초기화
        let moka_cache: Cache<Uuid, Uuid> = moka::future::CacheBuilder::new(10_000)
            .time_to_live(Duration::from_secs(
                configuration.application.token_expiration_seconds as u64,
            ))
            .time_to_idle(Duration::from_hours(1))
            .build();

        // `AppState`` 생성
        let app_state = app_state::AppState::new(
            connection_pool,
            email_client,
            configuration.application.base_url,
            configuration.application.token_expiration_seconds,
            private_key,
            public_key,
            moka_cache,
        );

        // `Router` 생성
        let app_router = get_app_router(app_state);

        Ok(Self {
            listener,
            app_router,
        })
    }

    pub async fn port(&self) -> Result<u16, std::io::Error> {
        self.listener.local_addr().map(|addr| addr.port())
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.app_router).await
    }
}
