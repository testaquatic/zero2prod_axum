use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;

use crate::routes::{health_check, subscribe};

/// axum 서버를 시작하고, 지정된 리스너에서 요청을 처리한다.
pub fn run(
    listener: tokio::net::TcpListener,
    db_pool: DatabaseConnection,
) -> impl Future<Output = Result<(), std::io::Error>> {
    let db_pool = Arc::new(db_pool);
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(db_pool);
    axum::serve(listener, app).into_future()
}
