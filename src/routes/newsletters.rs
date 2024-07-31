use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    extract::{self},
    response::{IntoResponse, Response},
    Extension,
};

use crate::{
    authentication::{basic_authentication, AuthError}, database::Z2PADB, domain::SubscriberEmail, email_client::EmailClient, settings::{DefaultDBPool, DefaultEmailClient}, utils::error_chain_fmt
};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

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
                Response::builder()
                    // 인증 오류에 대해 401을 반환한다.
                    .status(http::StatusCode::UNAUTHORIZED)
                    // http는 여러 잘 알려진 표준 HTTP 헤더의 이름에 관한 상수셋을 제공한다.
                    .header(http::header::WWW_AUTHENTICATE, r#"Basic realm="publish""#)
                    .body(Body::empty())
                    .unwrap()
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
pub async fn publish_newsletter(
    Extension(pool): Extension<Arc<DefaultDBPool>>,
    Extension(email_client): Extension<Arc<DefaultEmailClient>>,
    headers: http::HeaderMap,
    extract::Json(body): extract::Json<BodyData>,
) -> Result<Response, PublishError> {
    // 오류를 부풀리고 필요한 반환을 수행한다.
    let credentials = basic_authentication(&headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    // 발신자의 uuid를 확인한다.
    let user_id = credentials.validate_credentials( &pool).await
    // `AuthError`의 variant는 매핑했지만 전체 오류는 `PublishError` variant의 생성자들에 전달한다.
    // 이를 통해 미들웨어에 의해 오류가 기독될 때 톱레벨 래퍼의 컨텍스트가 유지되도록 보장한다.
    .map_err(|e| match e {
        AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
        AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into())
    })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    // 이메일을 보낼 구독자 목록을 생성한다.
    let subscribers = pool
        .get_confirmed_subscribers()
        .await
        .map_err(anyhow::Error::from)?;

    for subscriber in subscribers {
        let subscriber_email = SubscriberEmail::try_from(subscriber.email.clone())
            .map_err(|e| PublishError::UnexpectedError(anyhow::anyhow!("Invalid subscriberemail: {}", e)))?;
        email_client
            .send_email(
                &subscriber_email,
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .with_context(|| format!("Failed to send newsletter issue to {}", &subscriber.email))?;
    }
    Ok(http::StatusCode::OK.into_response())
}

