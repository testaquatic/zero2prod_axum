use std::time::Duration;

use axum::routing;
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app_state,
    configuration::{self},
    handler::{
        health_check::{self, health_check},
        subscriptions::{self, subscribe},
    },
    middleware::request_id::RequestIdLayer,
};

/// 앱 라우터를 생성한다.
/// todo: 권한을 가지고 있는 사용자만 접근 가능하게 만들기
pub fn get_app_router(pool: sqlx::PgPool) -> axum::Router {
    let openapi_router = get_swagger_router();
    let app_state = app_state::AppState::new(pool.clone());

    axum::Router::new()
        .route("/health_check", routing::get(health_check))
        .route("/subscriptions", routing::post(subscribe))
        .with_state(app_state)
        .merge(openapi_router)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(RequestIdLayer)
}

/// 스웨거 라우터를 생성한다.
fn get_swagger_router() -> axum::Router {
    let (router, mut api) = OpenApiRouter::new().split_for_parts();
    api.merge(health_check::HealthCheckApiDoc::openapi());
    api.merge(subscriptions::SubscriptionsApiDoc::openapi());

    router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
}

pub async fn run() -> Result<(), std::io::Error> {
    // 설정을 읽는다
    let configuration = configuration::get_configuration().expect("failed to read configuration");

    // 리스너 생성
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    tracing::info!("listening on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;

    // 데이터베이스 풀 생성
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // 앱 라우터
    let app = get_app_router(connection_pool);

    // 서버 실행
    axum::serve(listener, app).await
}
