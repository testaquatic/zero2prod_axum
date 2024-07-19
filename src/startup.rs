use std::{
    future::{Future, IntoFuture},
    sync::Arc,
};

use axum::{
    routing::{self},
    Router,
};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

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
        .with_state(Arc::new(pool))
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        );
    axum::serve(tcp_listener, app).into_future()
}
