use std::{
    collections::HashMap,
    future::{Future, IntoFuture},
};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Form, Router,
};
use tokio::net::TcpListener;

// curl -v http://127.0.0.1:8000 => Hello World!
async fn greet(Path(map): Path<HashMap<String, String>>) -> Response {
    let world = "World".to_string();
    let name = map.get("name").unwrap_or(&world);

    format!("Hello {}!", name).into_response()
}

// curl -v http://127.0.0.1:8000/health_check => 200 OK
async fn health_check() -> Response {
    StatusCode::OK.into_response()
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

// 올바르지 않은 입력 => 422 Unprocessable Entity
// 올바른 입력 => 200 OK
async fn subscribe(Form(_form): Form<FormData>) -> Response {
    StatusCode::OK.into_response()
}

// `run`을 `public`으로 마크해야 한다.
// `run`은 더 이상 바이너리 엔트리 포인트가 아니므로, proc-macro 주문 없이 async로 마크할 수 있다.
pub fn run(tcp_listener: TcpListener) -> impl Future<Output = Result<(), std::io::Error>> {
    let app = Router::new()
        .route("/", routing::get(greet))
        .route("/health_check", routing::get(health_check))
        // POST /subscriptions 요청에 대한 라우팅 테이블의 새 엔트리 포인트
        .route("/subscriptions", routing::post(subscribe));
    axum::serve(tcp_listener, app).into_future()
}
