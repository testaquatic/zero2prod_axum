use axum::http;

#[utoipa::path(
    description = "작동 상태를 확인한다. 정상적으로 작동하다면 200 OK를 전달한다.",
    summary = "작동 상태 확인",
    get,
    path = "/health_check",
    responses(
        (status = http::StatusCode::OK, description = "OK")
    )
)]
pub async fn health_check() -> http::StatusCode {
    http::StatusCode::OK
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(health_check))]
pub struct HealthCheckApiDoc;
