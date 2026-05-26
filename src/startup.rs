use std::{sync::Arc, time::Duration};

use axum::{Router, routing};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app_state,
    configuration::{self},
    email_client,
    handler::{
        health_check::{self, health_check},
        newsletter::{self, publish_newsletter},
        subscriptions::{self, subscribe},
        subscriptions_confirm::{self, confirm},
    },
    middleware::request_id::RequestIdLayer,
};

/// 앱 라우터를 생성한다.
/// todo: 권한을 가지고 있는 사용자만 접근 가능하게 만들기
pub fn get_app_router(app_state: Arc<app_state::AppState>) -> axum::Router {
    let openapi_router = get_swagger_router();

    axum::Router::new()
        .route("/health_check", routing::get(health_check))
        .route("/subscriptions", routing::post(subscribe))
        .route("/subscriptions/confirm", routing::get(confirm))
        .route("/newsletter", routing::post(publish_newsletter))
        .with_state(app_state)
        .merge(openapi_router)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(RequestIdLayer)
}

/// 스웨거 라우터를 생성한다.
fn get_swagger_router() -> axum::Router {
    let (router, mut api) = OpenApiRouter::new().split_for_parts();
    api.merge(health_check::HealthCheckApiDoc::openapi());
    api.merge(subscriptions::SubscriptionsApiDoc::openapi());
    api.merge(subscriptions_confirm::SubscriptionsConfirm::openapi());
    api.merge(newsletter::NewsletterOpenApiDoc::openapi());
    router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
}

pub fn get_connection_pool(configuration: &configuration::Settings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
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

pub async fn run() -> Result<(), std::io::Error> {
    // 설정을 읽는다
    let configuration = configuration::get_configuration().expect("failed to read configuration");

    let application = Application::build(configuration)
        .await
        .expect("failed to build application");

    // 서버 실행
    application.run_until_stopped().await
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

        // `AppState`` 생성
        let app_state = app_state::AppState::new(
            connection_pool,
            email_client,
            configuration.application.base_url,
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
