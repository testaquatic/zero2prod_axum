use std::sync::Arc;

use axum::{Json, extract::State, http};

use crate::{app_state::AppState, error::AppError};

/// 뉴스레터를 발행한다.
#[utoipa::path(
    description = "뉴스레터를 발행한다.",
    summary = "뉴스레터 발행",
    post,
    path = "/newsletter",
    params(
        ("Authorization" = String, Header, description = "Bearer authentication", example = "Bearer <token>"),
    ),
    security(
        ("bearerAuth" = []),
    ),
    request_body = BodyData,
    responses(
        (status = http::StatusCode::OK, description = "OK")
    )
)]
pub async fn publish_newsletter(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<BodyData>,
) -> Result<http::StatusCode, AppError> {
    app_state
        .newsletter_service
        .publish_newsletter(&app_state.pg_pool, &body, &app_state.email_client)
        .await?;
    Ok(http::StatusCode::OK)
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct BodyData {
    /// 제목
    pub title: String,
    /// 내용
    pub content: Content,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct Content {
    /// HTML 문서
    pub html: String,
    /// 일반 텍스트
    pub text: String,
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(publish_newsletter))]
pub struct NewsletterOpenApiDoc;
