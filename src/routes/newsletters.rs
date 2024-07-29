use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    extract::{self},
    response::{IntoResponse, Response},
    Extension,
};
use base64::Engine;
use secrecy::Secret;

use crate::{
    database::Z2PADB,
    domain::SubscriberEmail,
    email_client::EmailClient,
    settings::{DefaultDBPool, DefaultEmailClient},
    utils::error_chain_fmt,
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
    SubscriberEmailError(String),
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
            .record("error", format!("{}", &self))
            .record("error_detail", format!("{:?}", &self));

        match self {
            PublishError::SubscriberEmailError(err) => {
                tracing::warn!("A confirmed subscriber is using an invalid email address.\n{err}");
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            PublishError::UnexpectedError(err) => {
                tracing::error!(target : "Z2PA", error = %err, error_detail = ?err);
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            PublishError::AuthError(err) => {
                tracing::error!(target : "Z2PA", error = %err, error_detail = ?err);
                let response = Response::builder()
                    .status(http::StatusCode::UNAUTHORIZED)
                    // http는 여러 잘 알려진 표준 HTTP 헤더의 이름에 관한 상수셋을 제공한다.
                    .header(http::header::WWW_AUTHENTICATE, r#"Basic realm="publish""#)
                    .body(Body::empty())
                    .unwrap();
                // 인증 오류에 대해 401을 반환한다.
                response
            }
        }
    }
}

// 뉴스레터 발송을 담당하는 엔드포인트
pub async fn publish_newsletter(
    Extension(pool): Extension<Arc<DefaultDBPool>>,
    Extension(email_client): Extension<Arc<DefaultEmailClient>>,
    headers: http::HeaderMap,
    extract::Json(body): extract::Json<BodyData>,
) -> Result<Response, PublishError> {
    // 오류를 부풀리고 필요한 반환을 수행한다.
    let credentials = basic_authentication(&headers).map_err(PublishError::AuthError)?;
    let subscribers = pool
        .get_confirmed_subscribers()
        .await
        .map_err(anyhow::Error::from)?;

    for subscriber in subscribers {
        let subscriber_email = SubscriberEmail::try_from(subscriber.email.clone())
            .map_err(PublishError::SubscriberEmailError)?;
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

struct Credentials {
    username: String,
    password: Secret<String>,
}

fn basic_authentication(headers: &http::HeaderMap) -> Result<Credentials, anyhow::Error> {
    // 헤더값이 존재한다면 유효한 UTF8 문자열이어야 한다.
    let header_value = headers
        .get(http::header::AUTHORIZATION)
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The `Authorization` header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    // `:` 구분자를 사용해서 두 개의 세그먼트로 나눈다.
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or(anyhow::anyhow!(
            "A username must be provide in 'Basic' auth."
        ))?
        .to_string();
    let password = credentials
        .next()
        .ok_or(anyhow::anyhow!(
            "A password must be provided in 'Basic' auth."
        ))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
