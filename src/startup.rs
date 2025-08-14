use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{
        ConnectInfo, FromRef, MatchedPath, Request, connect_info::IntoMakeServiceWithConnectInfo,
    },
    middleware::AddExtension,
    routing::{get, post},
    serve::Serve,
};
use sea_orm::{DatabaseConnection, sqlx::postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{confirm, health_check, subscribe},
};

/// 상태를 저장한다.
/// [Derive Macro FromRef](https://docs.rs/axum/0.8.4/axum/extract/derive.FromRef.html) 이 문서를 참고로 했다.
#[derive(Clone, FromRef)]
struct AppState {
    db_pool: Arc<DatabaseConnection>,
    email_client: Arc<EmailClient>,
    base_url: Arc<ApplicationBaseUrl>,
}

/// `AppState`에서 사용하기 편하도록 래퍼타입을 적용한다.
pub struct ApplicationBaseUrl(pub String);

/// axum 서버를 시작하고, 지정된 리스너에서 요청을 처리한다.
pub fn run(
    listener: tokio::net::TcpListener,
    db_pool: DatabaseConnection,
    email_client: EmailClient,
    base_url: String,
) -> Server {
    let app = get_app(db_pool, email_client, base_url);
    tracing::info!("Server started");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
}

/// Router 인스턴스를 얻는다.
pub fn get_app(db_pool: DatabaseConnection, email_client: EmailClient, base_url: String) -> Router //IntoMakeServiceWithConnectInfo<Router, SocketAddr>
{
    let db_pool = Arc::new(db_pool);
    let email_client = Arc::new(email_client);
    let base_url = Arc::new(ApplicationBaseUrl(base_url));
    let app_state = AppState {
        db_pool,
        email_client,
        base_url,
    };

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .layer(
            // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs 이곳의 소스코드를 참고로 했다
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);
                // https://docs.rs/axum/latest/axum/struct.Router.html#method.into_make_service_with_connect_info 이 문서를 참고로 했다.
                let remote_addr = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|addr| addr.0);

                tracing::info_span!(
                    "zero2prod_axum", method = ?request.method(), matched_path, request_id = %uuid::Uuid::new_v4(), ?remote_addr
                )
            })
        )
        .with_state(app_state)
}

/// IDE의 도움을 받았다.
type Server = Serve<
    TcpListener,
    IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
    AddExtension<Router, ConnectInfo<SocketAddr>>,
>;

pub struct Applicaton {
    port: u16,
    server: Server,
}

impl Applicaton {
    /// 서버 인스턴스를 빌드한다.
    pub async fn build(configuration: &Settings) -> Result<Self, std::io::Error> {
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
        let port = listener.local_addr().unwrap().port();

        let base_url = configuration.application.base_url.clone();
        let server = run(listener, connection_pool, email_client, base_url);

        Ok(Self { port, server })
    }

    /// 포트 번호를 반환한다.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 서버가 중지되어야 값이 반환된다.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// `DatabaseSettings`로부터 `DatabaseConnection`을 얻는다.
pub fn get_connection_pool(configuration: &DatabaseSettings) -> DatabaseConnection {
    // https://www.sea-ql.org/sea-orm-cookbook/015-lazy-connection.html 이 문서를 참고로 했다.
    let pgpool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db());
    sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(pgpool)
}
