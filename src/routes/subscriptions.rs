use crate::{
    database::PostgresPool,
    domain::{InvalidNewSubscriber, NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::Postmark,
    error::Z2PAError,
    startup::ApplicationBaseUrl,
    utils::error_chain_fmt,
};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = InvalidNewSubscriber;
    fn try_from(form_data: FormData) -> Result<Self, Self::Error> {
        let new_subscriber = NewSubscriber::new(
            SubscriberEmail::try_from(form_data.email)?,
            SubscriberName::try_from(form_data.name)?,
        );
        Ok(new_subscriber)
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error(transparent)]
    ValidationErr(anyhow::Error),
    #[error(transparent)]
    UnexpectedErr(anyhow::Error),
}

impl From<Z2PAError> for SubscribeError {
    fn from(e: Z2PAError) -> Self {
        match e {
            Z2PAError::InvalidNewSubscriber(e) => SubscribeError::ValidationErr(e.into()),
            _ => SubscribeError::UnexpectedErr(e.into()),
        }
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, error_detail = ?self);
        tracing::Span::current()
            .record("error", tracing::field::display(&self))
            .record("error_detail", tracing::field::debug(&self));
        match self {
            // `form`이 유효하지 않으면 400을 빠르게 반환한다.
            SubscribeError::ValidationErr(_) => http::StatusCode::BAD_REQUEST,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

// `curl --request POST --data 'name=le%20guin' --verbose http://127.0.0.1:8000/subscriptions`
// => 422 Unprocessable Entity Form 직렬화 실패
// `curl --request POST --data 'email=thomas_mann@hotmail.com&name=Tom' --verbose http://127.0.0.1:8000/subscriptions`
// => 500 Internal Server Error 데이터 베이스 오류, 이메일 전송 실패
// => 200 OK 정상 작동
// `curl --request POST --data 'email=thomas_mannhotmail.com&name=Tom' --verbose http://127.0.0.1:8000/subscriptions`
// => 필드 검증 실패 => 400 Bad Request
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip_all,
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    State(pool): State<Arc<PostgresPool>>,
    // 앱 콘텍스트에서 이메일 클라인트를 받는다.
    State(email_client): State<Arc<Postmark>>,
    State(base_url): State<Arc<ApplicationBaseUrl>>,
    // axum의 특성상 Form은 마지막으로 가야 한다.
    Form(form): Form<FormData>,
) -> axum::response::Result<Response, SubscribeError> {
    // 반환을 `Ok`로 감싸야 한다.
    // `Result`는 `Ok`와 `Err`라는 두개의 변형을 갖는다.
    // 첫 번째는 성공, 두 번째는 실패를 의미한다.
    // `match` 구문을 사용해서 결과에 따라 무엇을 수행할지 선택한다.
    let new_subscriber = TryFrom::try_from(form)
        .map_err(|e: InvalidNewSubscriber| SubscribeError::ValidationErr(e.into()))?;

    // `?` 연산자는 투명하게 `Into` 트레이트를 호출한다.
    let subscription_token = pool
        .insert_subscriber(&new_subscriber)
        .await
        .map_err(Z2PAError::DatabaseError)?;

    // 이메일을 신규 가입자에게 전송한다.
    // 전송에 실패하면 `INTERNAL_SERVER_ERROR`를 반환한다.
    // 애플리케이션 url을 전달한다.
    email_client
        .send_confirmation_email(new_subscriber, &base_url.0, &subscription_token)
        .await
        .map_err(Z2PAError::EmailClientError)?;
    Ok(http::StatusCode::OK.into_response())
}
