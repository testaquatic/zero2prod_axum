use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract,
    response::{IntoResponse, Response},
    Extension,
};

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
                tracing::warn!("A confirmed subscriber is using an invalid email address.\n{err}")
            }
            PublishError::UnexpectedError(err) => {
                tracing::error!(target : "Z2PA", error = %err, error_detail = ?err)
            }
        }

        http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

// 뉴스레터 발송을 담당하는 엔드포인트
pub async fn publish_newsletter(
    Extension(pool): Extension<Arc<DefaultDBPool>>,
    Extension(email_client): Extension<Arc<DefaultEmailClient>>,
    extract::Json(body): extract::Json<BodyData>,
) -> Result<Response, PublishError> {
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
