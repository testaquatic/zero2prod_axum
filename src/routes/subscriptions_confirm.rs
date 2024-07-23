use std::sync::Arc;

use axum::{
    extract::Query,
    response::{IntoResponse, Response},
    Extension,
};
use http::StatusCode;

use crate::database::{postgres::PostgresPool, Zero2ProdAxumDatabase};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn confirm(
    Query(parameters): Query<Parameters>,
    Extension(pool): Extension<Arc<PostgresPool>>,
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
        // 존재하지 않는 코믄
        None => StatusCode::UNAUTHORIZED.into_response(),
        Some(subscriber_id) => {
            if pool.confirm_subscriber(subscriber_id).await.is_err() {
                return http::StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            http::StatusCode::OK.into_response()
        }
    }
}
