use std::{
    future::{Future, IntoFuture},
    sync::Arc,
};

use axum::{
    body::Body,
    routing::{self},
    Router,
};
use http::Request;
use tokio::net::TcpListener;
use tower_http::trace::{MakeSpan, TraceLayer};
use tracing::{Level, Span};

use crate::{
    routes::{health_check::health_check, root::root, subscriptions::subscribe},
    settings::DefaultDBPool,
};

// `run`을 `public`으로 마크해야 한다.
// `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
pub fn run(
    tcp_listener: TcpListener,
    pool: DefaultDBPool,
) -> impl Future<Output = Result<(), std::io::Error>> {
    let app = Router::new()
        .route("/", routing::get(root))
        .route("/health_check", routing::get(health_check))
        // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
        .route("/subscriptions", routing::post(subscribe))
        // Arc로 pool을 감싼다.
        .layer(TraceLayer::new_for_http().make_span_with(AddRequestID))
        // .layer(add_request_id::AddRequestID)
        .with_state(Arc::new(pool));
    axum::serve(tcp_listener, app).into_future()
}

// https://docs.rs/tower-http/0.5.2/src/tower_http/trace/make_span.rs.html#65-68의 코드를 참조했음
#[derive(Clone)]
struct AddRequestID;

impl MakeSpan<Body> for AddRequestID {
    fn make_span(&mut self, request: &Request<Body>) -> Span {
        tracing::span!(
            Level::ERROR,
            "request",
            request_id=%uuid::Uuid::new_v4().to_string(),
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            headers = ?request.headers()
        )
    }
}
