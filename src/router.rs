use std::sync::Arc;

use axum::{middleware, routing};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app_state::{self, AppState},
    handler::{
        health_check::{self, health_check},
        newsletter::{self, publish_newsletter},
        subscriptions::{self, subscribe},
        subscriptions_confirm::{self, confirm},
    },
    middleware::{credentials::ExtractCredentials, request_id::RequestIdLayer},
};

/// 앱 라우터를 생성한다.
/// todo: 권한을 가지고 있는 사용자만 접근 가능하게 만들기
pub fn get_app_router(app_state: Arc<app_state::AppState>) -> axum::Router {
    let openapi_router = get_swagger_router();
    let protected_router = get_protected_router(app_state.clone());

    axum::Router::new()
        .route("/health_check", routing::get(health_check))
        .route("/subscriptions", routing::post(subscribe))
        .route("/subscriptions/confirm", routing::get(confirm))
        .merge(protected_router)
        .with_state(app_state)
        .merge(openapi_router)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(RequestIdLayer)
}

pub fn get_protected_router(app_state: Arc<app_state::AppState>) -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/newsletter", routing::post(publish_newsletter))
        .route_layer(middleware::from_extractor_with_state::<
            ExtractCredentials,
            Arc<AppState>,
        >(app_state))
}

/// 스웨거 라우터를 생성한다.
fn get_swagger_router() -> axum::Router {
    let (router, mut api) = OpenApiRouter::new().split_for_parts();
    api.merge(health_check::HealthCheckApiDoc::openapi());
    api.merge(subscriptions::SubscriptionsApiDoc::openapi());
    api.merge(subscriptions_confirm::SubscriptionsConfirm::openapi());
    api.merge(newsletter::NewsletterOpenApiDoc::openapi());
    router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
}
