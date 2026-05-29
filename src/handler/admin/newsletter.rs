use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http,
    response::{IntoResponse, Response},
};

use crate::{
    app_state::AppState,
    domain::{extractor::TokenData, form_data::PostNewsletterFormData},
    error::AppError,
};

/// 뉴스레터를 발행한다.
#[tracing::instrument(name = "Publish a newsletter issue", skip_all, err(Debug))]
#[utoipa::path(
    description = "뉴스레터를 발행한다.\n
뉴스레터를 발행하려면 /admin/idempotency_key 에서 일회용 키를 발급받거나 무작위로 생성한 Uuid를 사용한다.",
    summary = "뉴스레터 발행",
    post,
    path = "/admin/newsletters",
    params(
        ("Authorization" = String, Header, description = "Bearer authentication", example = "Bearer <token>"),
    ),
    security(
        ("bearerAuth" = []),
    ),
    request_body = PostNewsletterFormData,
    responses(
        (status = http::StatusCode::OK, description = "OK")
    ),
    tags = ["Newsletter"]
)]
pub async fn publish_newsletter(
    State(app_state): State<Arc<AppState>>,
    token_data: TokenData,
    Json(body): Json<PostNewsletterFormData>,
) -> Result<Response, AppError> {
    // 저장한 응답이 있다면 일찍 반환한다.
    if let Some(saved_response) = app_state
        .newsletter_service
        .get_idempotency_response(&app_state, &body, &token_data)
        .await?
    {
        return Ok(saved_response.into_response());
    }

    app_state
        .newsletter_service
        .publish_newsletter(&app_state, &body)
        .await?;

    Ok(http::StatusCode::OK.into_response())
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(publish_newsletter))]
pub struct NewsletterOpenApiDoc;
