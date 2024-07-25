use crate::{
    database::Zero2ProdAxumDatabase,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::{EmailClient, Postmark},
    error::{Z2PAErrResponse, Z2PAError},
    settings::DefaultDBPool,
    startup::ApplicationBaseUrl,
};
use axum::{
    response::{IntoResponse, Response},
    Extension, Form,
};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = Z2PAError;
    fn try_from(form_data: FormData) -> Result<Self, Self::Error> {
        let new_subscriber = NewSubscriber::new(
            SubscriberEmail::try_from(form_data.email)?,
            SubscriberName::try_from(form_data.name)?,
        );
        Ok(new_subscriber)
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
    Extension(pool): Extension<Arc<DefaultDBPool>>,
    // 앱 콘텍스트에서 이메일 클라인트를 받는다.
    Extension(email_client): Extension<Arc<Postmark>>,
    Extension(base_url): Extension<Arc<ApplicationBaseUrl>>,
    // axum의 특성상 Form은 마지막으로 가야 한다.
    Form(form): Form<FormData>,
) -> axum::response::Result<Response, Z2PAErrResponse> {
    // 반환을 `Ok`로 감싸야 한다.
    // `Result`는 `Ok`와 `Err`라는 두개의 변형을 갖는다.
    // 첫 번째는 성공, 두 번째는 실패를 의미한다.
    // `match` 구문을 사용해서 결과에 따라 무엇을 수행할지 선택한다.
    let new_subscriber = match form.try_into() {
        Ok(new_subscriber) => new_subscriber,
        // `form`이 유효하지 않으면 400을 빠르게 반환한다.
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST.into_response()),
    };

    // `?` 연산자는 투명하게 `Into` 트레이트를 호출한다.
    let subscription_token = pool
        .insert_subscriber(&new_subscriber)
        .await
        .map_err(Z2PAErrResponse::StoreTokenError)?;

    // 이메일을 신규 가입자에게 전송한다.
    // 전송에 실패하면 `INTERNAL_SERVER_ERROR`를 반환한다.
    // 애플리케이션 url을 전달한다.
    if email_client
        .send_confirmation_email(new_subscriber, &base_url.0, &subscription_token)
        .await
        .is_err()
    {
        return Ok(http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    Ok(http::StatusCode::OK.into_response())
}
