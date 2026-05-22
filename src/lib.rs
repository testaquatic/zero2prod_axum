pub mod domain;

use axum::{Form, http, routing};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::SubscribeFormData;

/// 앱 라우터를 생성한다.
/// todo: 권한을 가지고 있는 사용자만 접근 가능하게 만들기
pub fn get_app_router() -> axum::Router {
    let openapi_router = get_swagger_router();

    axum::Router::new()
        .route("/health_check", routing::get(health_check))
        .route("/subscriptions", routing::post(subscribe))
        .merge(openapi_router)
}

/// 스웨거 라우터를 생성한다.
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

#[tokio::test]
async fn test_health_check() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let app = axum::Router::new().route("/health_check", routing::get(health_check));
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/health_check")
        .body(Body::empty())
        .expect("failed to build request");
    let response = app.oneshot(request).await.expect("failed to send request");

    assert_eq!(response.status(), http::StatusCode::OK);
}

#[utoipa::path(
    description = "구독 요청을 받는다.",
    summary = "구독 요청",
    post,
    path = "/subscriptions",
    request_body(content = inline(SubscribeFormData), content_type = "application/x-www-form-urlencoded"),
    responses(
      (status = http::StatusCode::OK, description = "OK"),
      (status = http::StatusCode::UNPROCESSABLE_ENTITY, description = "누락되거나 유효하지 않은 필드가 있을 때")
    )
)]
async fn subscribe(Form(form_data): Form<SubscribeFormData>) -> http::StatusCode {
    http::StatusCode::OK
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(health_check, subscribe))]
struct ApiDoc;

pub async fn run() -> Result<(), std::io::Error> {
    // 앱 라우터
    let app = get_app_router();

    // 리스너 생성
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;

    // 서버 실행
    axum::serve(listener, app).await
}
