use axum::{
    Form,
    extract::State,
    http::{self},
};

use crate::{app_state::AppState, domain::subscriber::SubscribeData, handler::error::AppError};

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
  request_body(content = inline(SubscribeData), content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = http::StatusCode::OK, description = "OK"),
    (status = http::StatusCode::UNPROCESSABLE_ENTITY, description = "누락되거나 유효하지 않은 필드가 있음"),
    (status = http::StatusCode::INTERNAL_SERVER_ERROR, description = "서버 내부 오류, 자세한 내용은 로그를 참고")
  )
)]
pub async fn subscribe(
    State(app_state): State<AppState>,
    Form(form_data): Form<SubscribeData>,
) -> Result<http::StatusCode, AppError> {
    app_state.subscribe_service.subscribe(&form_data).await?;

    Ok(http::StatusCode::OK)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(subscribe))]
pub struct SubscriptionsApiDoc;
