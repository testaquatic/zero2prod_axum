use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http,
};

use crate::{app_state::AppState, handler::error::AppError};

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct Parameters {
    /// 구독을 확인하는 일회용 토큰
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a new subscriber", skip_all)]
#[utoipa::path(
  description = "구독 요청을 확인한다.",
  summary = "구독 요청 확인",
  get,
  path = "/subscriptions/confirm",
  params(
    Parameters,
  ),
  responses(
    (status = http::StatusCode::OK, description = "OK"),
    (status = http::StatusCode::BAD_REQUEST, description = "요청 데이터 유효성 검증 실패"),
  )
)]
pub async fn confirm(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<Parameters>,
) -> Result<http::StatusCode, AppError> {
    app_state
        .subscribe_service
        .confirm(&params.subscription_token)
        .await?;
    Ok(http::StatusCode::OK)
}

#[derive(utoipa::OpenApi)]
#[openapi(paths(confirm))]
pub struct SubscriptionsConfirm;
