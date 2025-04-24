use axum::{
    Form, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tokio::net::TcpListener;

/// /health_check 핸들러이다.
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

/// /subscriptions 핸들러이다.
async fn subscribe(form: Form<FormData>) -> Response {
    StatusCode::OK.into_response()
}

/// `Router`를 얻는다.
fn get_app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}

/// listener를 얻으려면 `async`가 필요하다.
pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = get_app();

    axum::serve(listener, app).await
}
