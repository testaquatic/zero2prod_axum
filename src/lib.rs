use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

/// /health_check 핸들러이다.
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

/// `Router`를 얻는다.
fn get_app() -> Router {
    Router::new().route("/health_check", get(health_check))
}

/// listener를 얻으려면 `async`가 필요하다.
pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = get_app();

    axum::serve(listener, app).await
}
