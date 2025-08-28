use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json,
    extract::State,
    http::{self, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use secrecy::SecretString;

use crate::{
    authentication::{AuthError, Credentials, validate_credentials},
    domain::SubscriberEmail,
    email_client::EmailClient,
    routes::error_chain_fmt,
};
/// /newsletters POST 요청에 사용하는 핸들러이다.
/// 요청 본문의 형식은 [BodyData]를 참고로 한다.
/// HTTP 기본인증을 사용한다.
#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip_all,
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn publish_newsletter(
    State(pool): State<Arc<sea_orm::DatabaseConnection>>,
    State(email_client): State<Arc<EmailClient>>,
    headers: HeaderMap,
    Json(body): Json<BodyData>,
) -> Result<StatusCode, PublishError> {
    let credentials = basic_authentication(headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(user_id));
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
    #[error("Athentication failed")]
    AuthError(#[source] anyhow::Error),
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
            // 예상하지 못한 오류에 대해서 500 Internal server error 를 반환한다. I
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            // 인증 오류에 대해서 401 Unauthorized를 반환한다.
            PublishError::AuthError(_) => {
                let status_code = StatusCode::UNAUTHORIZED;
                let mut headers = HeaderMap::new();
                let Ok(header_value) = HeaderValue::from_str(r#"Basic realm="publish""#) else {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                };
                headers.insert(http::header::WWW_AUTHENTICATE, header_value);

                (status_code, headers).into_response()
            }
        }
    }
}

/// HTTP 기본인증의 헤더를 처리한다.
fn basic_authentication(headers: HeaderMap) -> Result<Credentials, anyhow::Error> {
    // 해더값이 존재한다면 유효한 UTF8 문자열이다.
    let header_value = headers
        .get(http::header::AUTHORIZATION)
        .context("The 'Authorization' header was missing.")?;
    let base64encoded_segment = header_value
        .to_str()?
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    // https://docs.rs/base64/latest/base64/ 이문서를 참고로 했다.
    let decoded_bytes = BASE64_STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    // ':' 구분자를 사용해서 두개의 세그먼트로 나눈다.
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: SecretString::from(password),
    })
}
