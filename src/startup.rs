use std::sync::Arc;

use crate::{
    email_client::Postmark,
    error::Z2PAError,
    routes::{confirm, health_check, home, login, login_form, publish_newsletter, subscribe},
    settings::{DefaultDBPool, DefaultEmailClient, Settings},
};
use axum::{
    body::Body,
    extract::FromRef,
    routing::{self},
    Router,
};
use base64::Engine;
use http::Request;
use secrecy::ExposeSecret;
use tokio::net::TcpListener;
use tower_http::trace::{
    DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer,
};
use tracing::{Level, Span};

#[derive(Clone)]
pub struct AppState {
    flash_config: axum_flash::Config,
    pool: Arc<DefaultDBPool>,
    email_client: Arc<DefaultEmailClient>,
    base_url: Arc<ApplicationBaseUrl>,
}

impl FromRef<AppState> for axum_flash::Config {
    fn from_ref(input: &AppState) -> Self {
        input.flash_config.clone()
    }
}

impl FromRef<AppState> for Arc<DefaultDBPool> {
    fn from_ref(input: &AppState) -> Self {
        input.pool.clone()
    }
}

impl FromRef<AppState> for Arc<DefaultEmailClient> {
    fn from_ref(input: &AppState) -> Self {
        input.email_client.clone()
    }
}

impl FromRef<AppState> for Arc<ApplicationBaseUrl> {
    fn from_ref(input: &AppState) -> Self {
        input.base_url.clone()
    }
}

impl AppState {
    pub fn new(
        flash_config_key: &str,
        pool: DefaultDBPool,
        email_client: DefaultEmailClient,
        base_url: String,
    ) -> Result<AppState, base64::DecodeError> {
        let key = axum_flash::Key::from(
            &base64::engine::general_purpose::STANDARD_NO_PAD.decode(flash_config_key)?,
        );
        let flash_config = axum_flash::Config::new(key);
        Ok(AppState {
            flash_config,
            pool: Arc::new(pool),
            email_client: Arc::new(email_client),
            base_url: Arc::new(ApplicationBaseUrl(base_url)),
        })
    }
}

pub struct Server {
    tcp_listener: TcpListener,
    app_state: AppState,
}

// 래퍼 타입을 정의해서 `subscriber` 핸들러에서 URL을 꺼낸다.
pub struct ApplicationBaseUrl(pub String);

impl Server {
    pub fn new(tcp_listener: TcpListener, app_state: AppState) -> Self {
        Self {
            tcp_listener,
            app_state,
        }
    }

    pub async fn build(settings: &Settings) -> Result<Server, anyhow::Error> {
        let tcp_listener = settings.application.get_listener().await?;
        let pool = settings
            .database
            .get_pool::<DefaultDBPool>()
            .await
            .map_err(Z2PAError::DatabaseError)?;
        // `settings`를 사용해서 `EmailClient`를 만든다.
        let email_client = settings.email_client.get_email_client::<Postmark>()?;

        let app_state = AppState::new(
            settings.application.hmac_secret.expose_secret(),
            pool,
            email_client,
            settings.application.base_url.clone(),
        )?;

        let server = Server::new(tcp_listener, app_state);

        Ok(server)
    }

    // `run`을 `public`으로 마크해야 한다.
    // `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
    // `run`, `email_client`를 위한 새로운 인자
    pub async fn run(self) -> Result<(), std::io::Error> {
        let app = Router::new()
            .route("/", routing::get(home))
            .route("/health_check", routing::get(health_check))
            .route("/login", routing::get(login_form).post(login))
            .route("/newsletters", routing::post(publish_newsletter))
            // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
            .route("/subscriptions", routing::post(subscribe))
            .route("/subscriptions/confirm", routing::get(confirm))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(AddRequestID)
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_failure(DefaultOnFailure::new().level(Level::ERROR))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .include_headers(true),
                    ),
            )
            .with_state(self.app_state);

        tracing::info!(name: "server", status = "Starting the server now", addr = ?self.tcp_listener.local_addr());
        axum::serve(self.tcp_listener, app).await?;
        tracing::info!(name: "server", status = "Server closed");

        Ok(())
    }
}

// https://docs.rs/tower-http/0.5.2/src/tower_http/trace/make_span.rs.html#65-68의 코드를 참조했음
#[derive(Clone)]
struct AddRequestID;

impl MakeSpan<Body> for AddRequestID {
    fn make_span(&mut self, request: &Request<Body>) -> Span {
        if let Some(from) = request.headers().get("host") {
            tracing::span!(
                Level::INFO,
                "Z2PA",
                from = ?from,
                request_id = %uuid::Uuid::new_v4(),
                error = tracing::field::Empty,
                error_detail = tracing::field::Empty,
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        } else {
            tracing::span!(
                Level::INFO,
                "Z2PA",
                from = tracing::field::Empty,
                request_id = %uuid::Uuid::new_v4(),
                error = tracing::field::Empty,
                error_detail = tracing::field::Empty,
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        }
    }
}
