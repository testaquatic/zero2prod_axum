use std::sync::Arc;

use crate::{
    authentication::{reject_anonymous_users, PgSessionStorage},
    database::PostgresPool,
    email_client::Postmark,
    error::Z2PAError,
    routes::{
        admin_dashboard, admin_publish_newsletter, admin_publish_newsletter_form, change_password,
        change_password_form, confirm, health_check, home, log_out, login, login_form,
        publish_newsletter_basic_auth, subscribe,
    },
    settings::Settings,
};
use axum::{
    body::Body,
    extract::FromRef,
    middleware::{self},
    routing::{self},
    Router,
};
use base64::Engine;
use http::Request;
use secrecy::{ExposeSecret, Secret};
use tokio::{net::TcpListener, signal, task::AbortHandle};
use tower_http::trace::{
    DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer,
};
use tower_sessions::SessionManagerLayer;
use tracing::{Level, Span};

// 래퍼 타입을 정의해서 `subscriber` 핸들러에서 URL을 꺼낸다.
pub struct ApplicationBaseUrl(pub String);

// `app`의 정적인 상태를 표시한다.
// 동적인 것인 `Extension`을 사용한다.
#[derive(Clone)]
pub struct AppState {
    flash_config: axum_flash::Config,
    pool: Arc<PostgresPool>,
    email_client: Arc<Postmark>,
    base_url: Arc<ApplicationBaseUrl>,
}

impl FromRef<AppState> for axum_flash::Config {
    fn from_ref(input: &AppState) -> Self {
        input.flash_config.clone()
    }
}

impl FromRef<AppState> for Arc<PostgresPool> {
    fn from_ref(input: &AppState) -> Self {
        input.pool.clone()
    }
}

impl FromRef<AppState> for Arc<Postmark> {
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
        flash_config_key: &Secret<String>,
        pool: PostgresPool,
        email_client: Postmark,
        base_url: &str,
    ) -> Result<AppState, anyhow::Error> {
        let key = axum_flash::Key::from(
            &base64::engine::general_purpose::STANDARD_NO_PAD
                .decode(flash_config_key.expose_secret())?,
        );

        let flash_config = axum_flash::Config::new(key).use_secure_cookies(true);
        Ok(AppState {
            flash_config,
            pool: Arc::new(pool),
            email_client: Arc::new(email_client),
            base_url: Arc::new(ApplicationBaseUrl(base_url.to_string())),
        })
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
                header = ?request.headers(),
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

pub struct Server {
    tcp_listener: TcpListener,
    app_state: AppState,
    session_storage: PgSessionStorage,
}

impl Server {
    pub fn new(
        tcp_listener: TcpListener,
        app_state: AppState,
        session_storage: PgSessionStorage,
    ) -> Server {
        Self {
            tcp_listener,
            app_state,
            session_storage,
        }
    }

    pub async fn build(settings: &Settings) -> Result<Server, anyhow::Error> {
        let tcp_listener = settings.application.get_listener().await?;
        let pool = PostgresPool::connect(settings.database.connect_options_with_db())
            .map_err(Z2PAError::DatabaseError)?;
        // `settings`를 사용해서 `EmailClient`를 만든다.
        let email_client = settings.email_client.get_email_client()?;

        let app_state = AppState::new(
            &settings.application.hmac_secret,
            pool.clone(),
            email_client,
            &settings.application.base_url,
        )?;

        let session_storage =
            PgSessionStorage::init(pool, settings.application.hmac_secret.clone()).await?;

        let server = Server::new(tcp_listener, app_state, session_storage);

        Ok(server)
    }

    // `run`을 `public`으로 마크해야 한다.
    // `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
    // `run`, `email_client`를 위한 새로운 인자
    // std::io::Error 대신 anyhow::Result이다.
    pub async fn run(self) -> Result<(), anyhow::Error> {
        let session_layer = SessionManagerLayer::new(self.session_storage.session_store)
            .with_private(
                self.session_storage
                    .key
                    .expose_secret()
                    .as_bytes()
                    .try_into()?,
            );
        let app = Router::new()
            .route("/", routing::get(home))
            .route("/health_check", routing::get(health_check))
            .route("/login", routing::get(login_form).post(login))
            .route("/newsletters", routing::post(publish_newsletter_basic_auth))
            // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
            .route("/subscriptions", routing::post(subscribe))
            .route("/subscriptions/confirm", routing::get(confirm))
            .nest(
                "/admin",
                Router::new()
                    .route("/logout", routing::post(log_out))
                    .route("/dashboard", routing::get(admin_dashboard))
                    .route(
                        "/password",
                        routing::get(change_password_form).post(change_password),
                    )
                    .route(
                        "/newsletters",
                        routing::get(admin_publish_newsletter_form).post(admin_publish_newsletter),
                    )
                    .layer(middleware::from_fn(reject_anonymous_users)),
            )
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
            .layer(session_layer)
            .with_state(self.app_state);

        tracing::info!(name: "server", status = "Starting the server now", addr = ?self.tcp_listener.local_addr());
        axum::serve(self.tcp_listener, app)
            .with_graceful_shutdown(shutdown_signal(
                self.session_storage.abort_handle.abort_handle(),
            ))
            .await?;
        tracing::info!(name: "server", status = "Server closed");

        Ok(())
    }
}

// https://github.com/maxcountryman/tower-sessions-stores/tree/main/sqlx-store 코드를 참조했다.
// 종료시 정리 코드를 실행한다.
async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::Pending::<()>();

    tokio::select! {
        _ = ctrl_c => deletion_task_abort_handle.abort(),
        _ = terminate => deletion_task_abort_handle.abort(),
    }
}
