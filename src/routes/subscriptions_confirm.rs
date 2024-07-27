use std::sync::Arc;

use axum::{
    extract::Query,
    response::{IntoResponse, Response},
    Extension,
};
use http::StatusCode;

use crate::{database::Z2PADB, settings::DefaultDBPool};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

// 200 => 정상 작동
// 401 => 유효하지 않은 토큰
// 500 => 내부 서버(데이터 베이스 등) 오류
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    Query(parameters): Query<Parameters>,
    Extension(pool): Extension<Arc<DefaultDBPool>>,
) -> Response {
    let id = match pool
        .as_ref()
        .get_subscriber_id_from_token(&parameters.subscription_token)
        .await
    {
        Ok(id) => id,
        Err(_) => return http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    match id {
        // 존재하지 않는 토큰
        None => {
            tracing::error!(
            name: "Invalid token",
            msg = %"Invalid token.",
            token = %parameters.subscription_token
            );
            StatusCode::UNAUTHORIZED.into_response()
        }
        Some(subscriber_id) => {
            if pool.confirm_subscriber(subscriber_id).await.is_err() {
                return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            http::StatusCode::OK.into_response()
        }
    }
}
