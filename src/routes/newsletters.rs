use std::sync::Arc;

use anyhow::Context;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

use crate::{domain::SubscriberEmail, email_client::EmailClient, routes::error_chain_fmt};
/// /newsletters POST 요청에 사용하는 핸들러이다.
/// 요청 본문의 형식은 [BodyData]를 참고로 한다.
pub async fn publish_newsletter(
    State(pool): State<Arc<sea_orm::DatabaseConnection>>,
    State(email_client): State<Arc<EmailClient>>,
    Json(body): Json<BodyData>,
) -> Result<StatusCode, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => email_client
                .send_email(
                    &subscriber.email,
                    &body.title,
                    &body.content.html,
                    &body.content.text,
                )
                .await
                .with_context(|| {
                    format!("Failed to send newsletter issue to {}", subscriber.email)
                })?,
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber.Their stored contact details are invalid."
                )
            }
        }
    }
    Ok(StatusCode::OK)
}

/// /newsletters POST 요청에 사용하는 json을 나타낸다.
/// ```json
/// {
///     "title": "Newsletter title",
///     "content": {
///         "text": "Newsletter body as plain text",
///         "html": "<p>Newsletter body as HTML</p>"
///     }
/// }
/// ```
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

/// 확인된 구독자를 나타내는 구조체이다.
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

/// 데이터베이스에서 확인된 모든 구독자 리스트를 꺼낸다.
#[tracing::instrument(name = "Get confirmed subscribers", skip_all)]
async fn get_confirmed_subscribers(
    pool: &sea_orm::DatabaseConnection,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    /// sea_orm에서 사용하기 위한 타입
    /// 데이터베이스에 저장한 이메일을 표현한다.
    #[derive(sea_orm::FromQueryResult)]
    struct Row {
        email: String,
    }

    let confirmed_subscribers = entities::subscriptions::Entity::find()
        .select_only()
        .column(entities::subscriptions::Column::Email)
        .filter(entities::subscriptions::Column::Status.eq("confirmed"))
        .into_model::<Row>()
        .all(pool)
        .await?
        .into_iter()
        .map(|r| {
            SubscriberEmail::parse(r.email)
                .map(|email| ConfirmedSubscriber { email })
                .map_err(|error| anyhow::anyhow!(error))
        })
        .collect();

    Ok(confirmed_subscribers)
}

/// publish_newsletter에서 반환하는 오류이다.
#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error: {self:?}");
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
