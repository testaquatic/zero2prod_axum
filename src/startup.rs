use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ ConnectInfo, MatchedPath, Request}, routing::{get, post}, Router
};
use sea_orm::DatabaseConnection;
use tower_http::trace::TraceLayer;

use crate::routes::{health_check, subscribe};

/// axum 서버를 시작하고, 지정된 리스너에서 요청을 처리한다.
pub async fn run(
    listener: tokio::net::TcpListener,
    db_pool: DatabaseConnection,
) -> Result<(), std::io::Error> {
    let db_pool = Arc::new(db_pool);
    let app = get_app(db_pool);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await
}

/// Router 인스턴스를 얻는다.
pub fn get_app(db_pool: Arc<DatabaseConnection>) -> Router//IntoMakeServiceWithConnectInfo<Router, SocketAddr>
{
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(
            // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/src/main.rs 이곳의 소스코드를 참고로 했다
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);
                // https://docs.rs/axum/latest/axum/struct.Router.html#method.into_make_service_with_connect_info 이 문서를 참고로 했다.
                let remote_addr = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|addr| addr.0);           

                tracing::info_span!(
                    "zero2prod_axum", method = ?request.method(), matched_path, request_id = %uuid::Uuid::new_v4(), ?remote_addr
                )
            }),
        )
        .with_state(db_pool)

}
