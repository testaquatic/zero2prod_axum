use std::sync::Arc;

use axum::{
    body::Body,
    routing::{self},
    Router,
};
use http::Request;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer};
use tracing::{Level, Span};

use crate::{
    email_client::EmailClient,
    routes::{health_check, root, subscribe},
    settings::DefaultDBPool,
};

pub struct Server {
    tcp_listener: TcpListener,
    pool: DefaultDBPool,
    email_client: EmailClient,
}

impl Server {
    pub fn new(tcp_listener: TcpListener, pool: DefaultDBPool, email_client: EmailClient) -> Self {
        Self {
            tcp_listener,
            pool,
            email_client,
        }
    }

    // `run`을 `public`으로 마크해야 한다.
    // `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
    pub async fn run(self) -> Result<(), std::io::Error> {
        let pool = Arc::new(self.pool);
        let email_client = Arc::new(self.email_client);
        let app = Router::new()
            .route("/", routing::get(root))
            .route("/health_check", routing::get(health_check))
            // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
            .route("/subscriptions", routing::post(subscribe))
            // Arc로 pool을 감싼다.
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(AddRequestID)
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .with_state(pool.clone())
            .with_state(email_client.clone());
        axum::serve(self.tcp_listener, app).await
    }
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
