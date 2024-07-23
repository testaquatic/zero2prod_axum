use std::sync::Arc;

use crate::{
    email_client::EmailClient,
    error::Zero2ProdAxumError,
    routes::{confirm, health_check, root, subscribe},
    settings::{DefaultDBPool, Settings},
};
use axum::{
    body::Body,
    routing::{self},
    Extension, Router,
};
use http::Request;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer};
use tracing::{Level, Span};

pub struct Server {
    tcp_listener: TcpListener,
    pool: DefaultDBPool,
    email_client: EmailClient,
    base_url: String,
}

// 래퍼 타입을 정의해서 `subscriber` 핸들러에서 URL을 꺼낸다.
pub struct ApplicationBaseUrl(pub String);

impl Server {
    pub fn new(
        tcp_listener: TcpListener,
        pool: DefaultDBPool,
        email_client: EmailClient,
        base_url: String,
    ) -> Self {
        Self {
            tcp_listener,
            pool,
            email_client,
            base_url,
        }
    }

    pub async fn build(settings: &Settings) -> Result<Server, Zero2ProdAxumError> {
        let tcp_listener = settings.application.get_listener().await?;
        let pool = settings.database.get_pool().await?;
        // `settings`를 사용해서 `EmailClient`를 만든다.
        let email_client = settings.email_client.get_email_client()?;

        let server = Server::new(
            tcp_listener,
            pool,
            email_client,
            settings.application.base_url.clone(),
        );

        Ok(server)
    }

    // `run`을 `public`으로 마크해야 한다.
    // `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
    // `run`, `email_client`를 위한 새로운 인자
    pub async fn run(self) -> Result<(), std::io::Error> {
        // Arc로 pool을 감싼다.
        let pool = Arc::new(self.pool);
        let email_client = Arc::new(self.email_client);
        let base_url = Arc::new(ApplicationBaseUrl(self.base_url));

        let subscriptions_method_router = routing::post(subscribe).layer(
            ServiceBuilder::new()
                .layer(Extension(email_client.clone()))
                .layer(Extension(base_url.clone())),
        );

        let app = Router::new()
            .route("/", routing::get(root))
            .route("/health_check", routing::get(health_check))
            // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
            .route("/subscriptions", subscriptions_method_router)
            .route("/subscriptions/confirm", routing::get(confirm))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(AddRequestID)
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .layer(Extension(pool.clone()));

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
