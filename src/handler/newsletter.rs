use std::sync::Arc;

use axum::{Json, extract::State, http};

use crate::{app_state::AppState, domain::form_data::PostNewsletterFormData, error::AppError};

/// 뉴스레터를 발행한다.
#[tracing::instrument(name = "Publish a newsletter issue", skip_all, err(Debug))]
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
    request_body = PostNewsletterFormData,
    responses(
        (status = http::StatusCode::OK, description = "OK")
    ),
    tags = ["Newsletter"]
)]
pub async fn publish_newsletter(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<PostNewsletterFormData>,
) -> Result<http::StatusCode, AppError> {
    app_state
        .newsletter_service
        .publish_newsletter(&app_state, &body)
        .await?;
    Ok(http::StatusCode::OK)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(publish_newsletter))]
pub struct NewsletterOpenApiDoc;
