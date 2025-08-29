use std::{net::SocketAddr, sync::Arc, time};

use anyhow::Context;
use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, FromRef, Request},
    routing::{get, post},
};
use cookie::Key;
use sea_orm::{DatabaseConnection, sqlx::postgres::PgPoolOptions};
use secrecy::{ExposeSecret, SecretString};
use tokio::{net::TcpListener, signal, task::AbortHandle};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{CachingSessionStore, ExpiredDeletion, SessionManagerLayer};
use tower_sessions_moka_store::MokaStore;
use tower_sessions_sqlx_store::PostgresStore;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{
        admin_dashbaord, confirm, health_check, hmac_check, home, login, login_form,
        publish_newsletter, subscribe,
    },
};

/// 상태를 저장한다.
/// [Derive Macro FromRef](https://docs.rs/axum/0.8.4/axum/extract/derive.FromRef.html) 이 문서를 참고로 했다.
#[derive(Clone, FromRef)]
struct AppState {
    db_pool: Arc<DatabaseConnection>,
    email_client: Arc<EmailClient>,
    base_url: Arc<ApplicationBaseUrl>,
    hmac_secret: Arc<HmacSecret>,
    index_html: Arc<IndexHtml>,
}

#[derive(Clone)]
pub struct HmacSecret(pub SecretString);

/// 자주 사용하므로 미리 캐싱해 놓는다.
/// 대신에 업데이트하려면 서버를 중지해야 한다.
pub struct IndexHtml {
    pub pub_html: String,
    pub admin_html: String,
}

impl AppState {
    /// `AppState`를 생성한다.
    pub fn new(
        db_pool: DatabaseConnection,
        email_client: EmailClient,
        base_url: String,
        hmac_secret: SecretString,
        index_html: IndexHtml,
    ) -> Self {
        Self {
            db_pool: Arc::new(db_pool),
            email_client: Arc::new(email_client),
            base_url: Arc::new(ApplicationBaseUrl(base_url)),
            hmac_secret: Arc::new(HmacSecret(hmac_secret)),
            index_html: Arc::new(index_html),
        }
    }
}

/// `AppState`에서 사용하기 편하도록 래퍼타입을 적용한다.
pub struct ApplicationBaseUrl(pub String);

/// `zero2prod_axum` 실행을 위한 구조체
pub struct Applicaton {
    listener: TcpListener,
    router: Router,
    pg_session_store: PostgresStore,
}

impl Applicaton {
    /// Settings로부터 Applicaton을 얻는다.
    /// 서버 프로그램을 실행하려면 Applicaton::run을 이용한다.
    pub async fn from_settings(configuration: &Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = tokio::net::TcpListener::bind(address).await?;

        let base_url = configuration.application.base_url.clone();

        let index_html = IndexHtml {
            pub_html: std::fs::read_to_string("web/public/dist/index.html")?,
            admin_html: std::fs::read_to_string("web/admin/dist/index.html")?,
        };

        let app_state = AppState::new(
            connection_pool,
            email_client,
            base_url,
            configuration.application.hmac_secret.clone(),
            index_html,
        );

        // https://github.com/maxcountryman/tower-sessions-stores/tree/main/sqlx-store 이 문서를 참고로 했다.
        let pg_session_store =
            PostgresStore::new(app_state.db_pool.get_postgres_connection_pool().clone());

        create_app(app_state, pg_session_store, listener).await
    }

    /// 포트 번호를 반환한다.
    pub fn port(&self) -> u16 {
        self.listener
            .local_addr()
            .expect("Failed to get local address")
            .port()
    }

    /// 서버를 실행한다.
    pub async fn run(self) -> Result<(), anyhow::Error> {
        // 세션저장소와 관련한 작업을 한다.
        self.pg_session_store
            .migrate()
            .await
            .context("Failed to migrate session store")?;
        let deletion_task = tokio::task::spawn(
            self.pg_session_store
                .continuously_delete_expired(time::Duration::from_secs(60)),
        );

        axum::serve(
            self.listener,
            self.router
                // 복잡한 타입에 대한 설명을 피하기 위해서 여기에 넣었다.
                // 논리의 흐름상 어색하다.
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await?;

        deletion_task.await??;

        Ok(())
    }
}

/// Application을 만든다.
async fn create_app(
    app_state: AppState,
    pg_session_store: PostgresStore,
    listener: TcpListener,
) -> Result<Applicaton, anyhow::Error> {
    // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs 이곳의 소스코드를 참고로 했다
    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
        let request_uri = request.uri().to_string();
        // https://docs.rs/axum/latest/axum/struct.Router.html#method.into_make_service_with_connect_info 이 문서를 참고로 했다.
        let remote_addr = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|addr| addr.0.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        tracing::info_span!(
            "zero2prod_axum", request_id = %uuid::Uuid::new_v4(), method = ?request.method(), request_uri, ?remote_addr
        )
    });

    // https://docs.rs/tower-sessions/latest/tower_sessions/struct.CachingSessionStore.html 이 문서를 참고로 했다.
    let session_store =
        CachingSessionStore::new(MokaStore::new(Some(2000)), pg_session_store.clone());
    let secret_key = Key::from(app_state.hmac_secret.0.expose_secret().as_bytes());
    // https://docs.rs/tower-sessions/latest/tower_sessions/service/struct.SessionManagerLayer.html#method.with_private 이 문서를 참고로 했다.
    let session_layer = SessionManagerLayer::new(session_store).with_private(secret_key);

    let router = Router::new()
        .route("/health_check", get(health_check))
        .nest(
            "/admin",
            Router::new()
                .route("/dashboard", get(admin_dashbaord))
                .fallback_service(ServeDir::new("web/admin/dist")),
        )
        .nest(
            "/subscriptions",
            Router::new()
                .route("/", post(subscribe))
                .route("/confirm", get(confirm)),
        )
        .route("/newsletters", post(publish_newsletter))
        .route("/home", get(home))
        .route("/login", get(login_form).post(login))
        .nest("/check", Router::new().route("/hmac", post(hmac_check)))
        // https://github.com/tokio-rs/axum/tree/main/examples/static-file-server 이 문서를 참고로 했다.
        .fallback_service(ServeDir::new("web/public/dist"))
        .layer(ServiceBuilder::new().layer(trace_layer))
        .layer(session_layer)
        .with_state(app_state);

    Ok(Applicaton {
        listener,
        router,
        pg_session_store,
    })
}

/// `DatabaseSettings`로부터 `DatabaseConnection`을 얻는다.
pub fn get_connection_pool(configuration: &DatabaseSettings) -> DatabaseConnection {
    // https://www.sea-ql.org/sea-orm-cookbook/015-lazy-connection.html 이 문서를 참고로 했다.
    let pgpool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db());
    sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(pgpool)
}

/// 시그널 핸들러이다.
async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
