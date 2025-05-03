use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request, connect_info::IntoMakeServiceWithConnectInfo},
    middleware::AddExtension,
    routing::{get, post},
    serve::Serve,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::Span;
use uuid::Uuid;

use crate::{
    configuration::{DatabaseSettings, Settings},
    database::ZPgPool,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

/// 그냥 경고 메시지의 타입을 복사했다.
/// 여기에서 포트 추출이 바로 가능하다.
type Server = Serve<
    tokio::net::TcpListener,
    Service,
    AddExtension<Router, ConnectInfo<std::net::SocketAddr>>,
>;

type Service = IntoMakeServiceWithConnectInfo<Router, SocketAddr>;

/// 라우터의 상태를 표현한다.
/// 정적인 데이터를 넣는다.
pub struct RouterState {
    pub z_pgpool: ZPgPool,
    pub email_client: EmailClient,
}

impl RouterState {
    pub fn new(z_pgpool: ZPgPool, email_client: EmailClient) -> Self {
        Self {
            z_pgpool,
            email_client,
        }
    }
}

/// `Router`를 얻는다.
fn get_service(router_state: RouterState) -> Service {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(make_span)))
        .with_state(Arc::new(router_state))
        .into_make_service_with_connect_info::<SocketAddr>()
}

/// 스팬을 생성한다.
fn make_span(request: &Request<Body>) -> Span {
    let remote_addr = request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|connect_info| &connect_info.0);
    tracing::info_span!(
        "http-request",
        request_id = %Uuid::new_v4(),
        method = %request.method(),
        path = %request.uri(),
        remote_addr = ?remote_addr,
    )
}

/// 새롭게 만들어진 서버 인스턴스
/// 타입 정의가 복잡하므로 추상화한다.
pub struct Application {
    server: Server,
}

impl Application {
    /// 서버 인스턴스를 생성한다.
    #[tracing::instrument(name = "build application", skip_all)]
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let z_pg_pool = get_z_pgpool(&configuration.database).await;
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        )
        .expect("Failed to create email client.");
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let router_state = RouterState::new(z_pg_pool.clone(), email_client);
        let server = run(listener, router_state);

        Ok(Application { server })
    }

    /// 포트 번호를 추출한다.
    pub fn port(&self) -> Result<u16, std::io::Error> {
        self.server.local_addr().map(|addr| addr.port())
    }

    /// 애플리 케이션이 중지 됐을 때 값을 반환한다.
    #[tracing::instrument(name = "run application", skip_all)]
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// ZPgPool을 얻는다.
pub async fn get_z_pgpool(database_settings: &DatabaseSettings) -> ZPgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(database_settings.with_db())
        .into()
}

/// listener를 얻으려면 `async`가 필요하다.
/// 서버 인스턴스를 얻는다.
pub fn run(listener: TcpListener, router_state: RouterState) -> Server {
    axum::serve(listener, get_service(router_state))
}
