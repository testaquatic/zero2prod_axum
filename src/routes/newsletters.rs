use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    extract::{self},
    response::{IntoResponse, Response},
    Extension,
};
use secrecy::ExposeSecret;
use sha3::Digest;

use crate::{
    database::Z2PADB,
    domain::SubscriberEmail,
    email_client::EmailClient,
    settings::{DefaultDBPool, DefaultEmailClient},
    utils::{basic_authentication, error_chain_fmt, Credentials},
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
    #[error("{0}")]
    SubscriberEmailErr(String),
    #[error(transparent)]
    UnexpectedErr(#[from] anyhow::Error),
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
            PublishError::SubscriberEmailErr(err) => {
                tracing::warn!("A confirmed subscriber is using an invalid email address.\n{err}");
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            // => 500
            PublishError::UnexpectedErr(err) => {
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

#[tracing::instrument(name = "Publish a newsletter issue", skip_all, fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
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
    let user_id = validate_credentials(&credentials, &pool).await?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    // 이메일을 보낼 구독자 목록을 생성한다.
    let subscribers = pool
        .get_confirmed_subscribers()
        .await
        .map_err(anyhow::Error::from)?;

    for subscriber in subscribers {
        let subscriber_email = SubscriberEmail::try_from(subscriber.email.clone())
            .map_err(PublishError::SubscriberEmailErr)?;
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

// 발신자를 확인하고 발신자의 uuid를 반환한다.
async fn validate_credentials(
    credentials: &Credentials,
    pool: &DefaultDBPool,
) -> Result<uuid::Uuid, PublishError> {
    let password_hash = sha3::Sha3_256::digest(credentials.password.expose_secret().as_bytes());
    // 소문자 16진수 인코딩
    let password_hash = format!("{password_hash:x}");
    let user_id = pool
        .validate_credentials(&credentials.username, &password_hash)
        .await
        .context("Failed to perform a query to validate credentials.")
        .map_err(PublishError::UnexpectedErr)?;

    user_id
        .ok_or(anyhow::anyhow!("Invalid username or password."))
        .map_err(PublishError::AuthError)
}
