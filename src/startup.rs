use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request},
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::Span;
use uuid::Uuid;

use crate::{
    database::ZPgPool,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

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
fn get_router(z_pgpool: ZPgPool, email_client: EmailClient) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(make_span)))
        .with_state(Arc::new(RouterState::new(z_pgpool, email_client)))
}

/// 스팬을 생성한다.
fn make_span(request: &Request<Body>) -> Span {
    let request_id = Uuid::new_v4();
    let remote_addr = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|socketaddr| socketaddr.to_string())
        .unwrap_or("UNKNOWN".to_string());
    tracing::info_span!(
        "http-request",
        %request_id,
        method = %request.method(),
        path = %request.uri(),
        remote_addr = %remote_addr,
    )
}

/// listener를 얻으려면 `async`가 필요하다.
/// 웹서버를 실행한다.
#[tracing::instrument(name = "Server", skip_all)]
pub async fn run(
    listener: TcpListener,
    z_pgpool: ZPgPool,
    email_client: EmailClient,
) -> Result<(), std::io::Error> {
    let app = get_router(z_pgpool, email_client);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
