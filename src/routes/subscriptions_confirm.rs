use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response, Result},
};

use crate::{
    database::{Z2PADBError, Z2PADB},
    settings::DefaultDBPool,
};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfrimError {
    #[error(transparent)]
    DatabaseError(Z2PADBError),
    #[error("Invalid token: {token}")]
    TokenError { token: String },
}

impl IntoResponse for ConfrimError {
    fn into_response(self) -> Response {
        match self {
            ConfrimError::DatabaseError(e) => {
                tracing::error!("{:?}", e);
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ConfrimError::TokenError { token } => {
                tracing::Span::current()
                    .record("error", "Invalid token")
                    .record("error_detail", &format!("{} is a invalid token.", &token));
                tracing::error!("Invalid token: {}", token);
                http::StatusCode::UNAUTHORIZED.into_response()
            }
        }
    }
}

// 200 => 정상 작동
// 401 => 유효하지 않은 토큰
// 500 => 내부 서버(데이터 베이스 등) 오류
#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    Query(parameters): Query<Parameters>,
    State(pool): State<Arc<DefaultDBPool>>,
) -> Result<Response, ConfrimError> {
    let id = pool
        .as_ref()
        .get_subscriber_id_from_token(&parameters.subscription_token)
        .await
        .map_err(ConfrimError::DatabaseError)?;
    match id {
        // 존재하지 않는 토큰
        None => {
            let token = parameters.subscription_token;
            Err(ConfrimError::TokenError { token })
        }
        Some(subscriber_id) => {
            pool.confirm_subscriber(subscriber_id)
                .await
                .map_err(ConfrimError::DatabaseError)?;
            Ok(http::StatusCode::OK.into_response())
        }
    }
}
