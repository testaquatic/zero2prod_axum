use axum::{http, routing};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    let app = get_app_router();

    let listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("failed to bind address");

    axum::serve(listener, app).await.expect("axum server error");
}

fn get_app_router() -> axum::Router {
    let openapi_router = get_swagger_router();

    axum::Router::new()
        .route("/health_check", routing::get(health_check))
        .merge(openapi_router)
}

fn get_swagger_router() -> axum::Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi()).split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
}

#[utoipa::path(
    description = "작동 상태를 확인한다. 정상적으로 작동하다면 200 OK를 전달한다.",
    summary = "작동 상태 확인",
    get,
    path = "/health_check",
    responses(
        (status = http::StatusCode::OK, description = "OK")
    )
)]
async fn health_check() -> http::StatusCode {
    http::StatusCode::OK
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(health_check))]
struct ApiDoc;
