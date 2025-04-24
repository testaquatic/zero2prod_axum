use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::routes::{health_check, subscribe};

/// `Router`를 얻는다.
fn get_app(db_pool: PgPool) -> Router {
    let db_pool = Arc::new(db_pool);
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(db_pool)
}

/// listener를 얻으려면 `async`가 필요하다.
pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<(), std::io::Error> {
    let app = get_app(db_pool);

    axum::serve(listener, app).await
}
