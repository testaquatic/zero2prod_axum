use axum::Json;
use uuid::Uuid;

use crate::{domain::response::IdempotencyKeyResponse, error::AppError};

#[tracing::instrument(name = "Get idempotency key", skip_all, err(Debug))]
#[utoipa::path(
    get,
    path = "/admin/idempotency_key",
    summary = "글쓰기 키 획득",
    description = "글쓰기 키를 서버에서 가져온다. 실체는 무작위로 생성한 Uuid이다.",
    responses(
      (status = http::StatusCode::OK, description = "OK", body = IdempotencyKeyResponse)
    ),
    security(("bearerAuth" = [])),
    tags = ["Newsletter"]
)]
pub async fn get_idempotency_key() -> Result<Json<IdempotencyKeyResponse>, AppError> {
    Ok(Json(IdempotencyKeyResponse {
        idempotency_key: Uuid::new_v4(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_idempotency_key))]
pub struct IdempotencyKeyOpenApiDoc;
