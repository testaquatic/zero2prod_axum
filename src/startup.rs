use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request},
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::Span;
use uuid::Uuid;

use crate::routes::{health_check, subscribe};

/// `Router`를 얻는다.
fn get_app(db_pool: PgPool) -> Router {
    let db_pool = Arc::new(db_pool);
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(db_pool)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(make_span)))
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
pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<(), std::io::Error> {
    let app = get_app(db_pool);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
