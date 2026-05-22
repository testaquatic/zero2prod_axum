use axum::{extract::Path, routing};
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
        .route("/hello", routing::get(greet))
        .route("/hello/{name}", axum::routing::get(greet_name))
        .merge(openapi_router)
}

fn get_swagger_router() -> axum::Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi()).split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
}

#[utoipa::path(
    description = "Hello World",
    summary = "Hello World",
    get,
    path = "/hello",
    responses(
        (status = 200, description = "Hello World", body = String, example = "Hello World!")
    )
)]
async fn greet() -> &'static str {
    "Hello World!"
}

#[utoipa::path(
    description = "Hello {name}",
    summary = "Hello {name}",
    get,
    path = "/hello/{name}",
    responses(
        (status = 200, description = "Hello {name}", body = String, example = "Hello {name}!")
    )
)]
async fn greet_name(Path(name): Path<String>) -> String {
    format!("Hello {}!", name)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(greet, greet_name))]
struct ApiDoc;
