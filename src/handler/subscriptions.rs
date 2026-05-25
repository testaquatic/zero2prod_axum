use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::{self},
};

use crate::{app_state::AppState, handler::error::AppError};

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name
    )
)]
#[utoipa::path(
  description = "구독 요청을 받는다.",
  summary = "구독 요청",
  post,
  path = "/subscriptions",
  request_body(content = inline(SubscribeFormData), content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = http::StatusCode::OK, description = "OK"),
    (status = http::StatusCode::UNPROCESSABLE_ENTITY, description = "누락되거나 유효하지 않은 필드가 있음"),
    (status = http::StatusCode::INTERNAL_SERVER_ERROR, description = "서버 내부 오류"),
    (status = http::StatusCode::BAD_REQUEST, description ="요청 데이터 유효성 검증 실패"),
  )
)]
pub async fn subscribe(
    State(app_state): State<Arc<AppState>>,
    Form(form_data): Form<SubscribeFormData>,
) -> Result<http::StatusCode, AppError> {
    app_state
        .subscribe_service
        .subscribe(&app_state.email_client, form_data, &app_state.base_url.0)
        .await?;

    Ok(http::StatusCode::OK)
}

/// 핸들러에 들어오는 가입 요청 데이터
#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct SubscribeFormData {
    pub name: String,
    pub email: String,
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(subscribe))]
pub struct SubscriptionsApiDoc;
