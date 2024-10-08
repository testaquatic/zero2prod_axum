use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    authentication::{basic_authentication, AuthError},
    database::{NextAction, PostgresPool},
    email_client::BodyData,
    idempotency::IdempotencyKey,
    utils::error_chain_fmt,
};

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
}

// 같은 로직을 사용해서 `Debug`에 대한 모든 오류 체인을 얻는다.
impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        tracing::Span::current()
            .record("error", tracing::field::display(&self))
            .record("error_detail", tracing::field::debug(&self));

        match self {
            // => 500
            PublishError::UnexpectedError(err) => {
                tracing::error!(target : "Z2PA", error = %err, error_detail = ?err);
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            // => 401
            PublishError::AuthError(err) => {
                tracing::error!(target : "Z2PA", error = %err, error_detail = ?err);

                (
                    // 인증 오류에 대해 401을 반환한다
                    http::StatusCode::UNAUTHORIZED,
                    // http는 여러 잘 알려진 표준 HTTP 헤더의 이름에 관한 상수셋을 제공한다.
                    [(http::header::WWW_AUTHENTICATE, r#"Basic realm="publish""#)],
                )
                    .into_response()
            }
        }
    }
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip_all,
    fields(
        username = tracing::field::Empty,
        user_id = tracing::field::Empty
    )
)]
// 뉴스레터 발송을 담당하는 엔드포인트
pub async fn publish_newsletter_basic_auth(
    State(pool): State<Arc<PostgresPool>>,
    headers: http::HeaderMap,
    extract::Json(body): extract::Json<BodyData>,
) -> Result<Response, PublishError> {
    // 오류를 부풀리고 필요한 반환을 수행한다.
    let credentials = basic_authentication(&headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    // 발신자의 uuid를 확인한다.
    let user_id = credentials
        .validate_credentials(&pool)
        .await
        // `AuthError`의 variant는 매핑했지만 전체 오류는 `PublishError` variant의 생성자들에 전달한다.
        // 이를 통해 미들웨어에 의해 오류가 기독될 때 톱레벨 래퍼의 컨텍스트가 유지되도록 보장한다.
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let next_action = pool
        .try_processing(
            &IdempotencyKey::try_from(Uuid::new_v4().to_string())
                .map_err(PublishError::UnexpectedError)?,
            user_id,
        )
        .await
        .map_err(|e| PublishError::UnexpectedError(e.into()))?;

    let mut transaciton = match next_action {
        NextAction::ReturnSavedResponse(_) => {
            return Err(PublishError::UnexpectedError(anyhow::anyhow!(
                "Unexpected uuid duplicate"
            )))
        }
        NextAction::StartProcessing(transaction) => transaction,
    };

    transaciton
        .schedule_newsletter_delivery(&body.title, &body.content.text, &body.content.html)
        .await
        .context("Failed to schedule newsletter delivery")
        .map_err(PublishError::UnexpectedError)?;
    transaciton
        .commit()
        .await
        .map_err(|e| PublishError::UnexpectedError(e.into()))?;

    Ok(http::StatusCode::ACCEPTED.into_response())
}
