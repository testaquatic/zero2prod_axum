use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{self},
};

use crate::{app_state::AppState, domain::form_data::SubscriptionFormData, error::AppError};

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name
    ),
    err(Debug)
)]
#[utoipa::path(
  description = "구독 요청을 받는다.",
  summary = "구독 요청",
  post,
  path = "/subscriptions",
  request_body(content = inline(SubscriptionFormData)),
  responses(
    (status = http::StatusCode::OK, description = "OK"),
    (status = http::StatusCode::UNPROCESSABLE_ENTITY, description = "누락되거나 유효하지 않은 필드가 있음"),
    (status = http::StatusCode::INTERNAL_SERVER_ERROR, description = "서버 내부 오류"),
    (status = http::StatusCode::BAD_REQUEST, description ="요청 데이터 유효성 검증 실패"),
  ),
  tags = ["Newsletter"]
)]
pub async fn subscribe(
    State(app_state): State<Arc<AppState>>,
    Json(form_data): Json<SubscriptionFormData>,
) -> Result<http::StatusCode, AppError> {
    app_state
        .subscriptions_service
        .subscribe(&app_state, form_data)
        .await?;

    Ok(http::StatusCode::OK)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(subscribe))]
pub struct SubscriptionsApiDoc;
