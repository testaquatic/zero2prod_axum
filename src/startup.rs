use std::future::{Future, IntoFuture};

use axum::{routing, Router};
use tokio::net::TcpListener;

use crate::routes::{health_check::health_check, root::root, subscriptions::subscribe};

// `run`을 `public`으로 마크해야 한다.
// `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
pub fn run(tcp_listener: TcpListener) -> impl Future<Output = Result<(), std::io::Error>> {
    let app = Router::new()
        .route("/", routing::get(root))
        .route("/health_check", routing::get(health_check))
        // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
        .route("/subscriptions", routing::post(subscribe));
    axum::serve(tcp_listener, app).into_future()
}
