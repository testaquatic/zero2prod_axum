use crate::{
    database::basic::Zero2ProdAxumDatabase,
    domain::{NewSubscriber, SubscriberName},
    settings::DefaultDBPool,
};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use http::StatusCode;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// `curl --request POST --data 'name=le%20guin' --verbose http://127.0.0.1:8000/subscriptions`
// => 422 Unprocessable Entity Form 직렬화 실패
// `curl --request POST --data 'email=thomas_mann@hotmail.com&name=Tom' --verbose http://127.0.0.1:8000/subscriptions`
// => 500 Internal Server Error 데이터 베이스 오류(중복된 이메일 등)
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
    State(pool): State<Arc<DefaultDBPool>>,
    // axum의 특성상 Form은 마지막으로 가야 한다.
    Form(form): Form<FormData>,
) -> Response {
    let new_subscriber = NewSubscriber {
        email: form.email,
        name: SubscriberName::parse(form.name),
    };
    // `Result`는 `Ok`와 `Err`라는 두개의 변형을 갖는다.
    // 첫 번째는 성공, 두 번째는 실패를 의미한다.
    // `match` 구문을 사용해서 결과에 따라 무엇을 수행할지 선택한다.
    match pool.as_ref().insert_subscriber(&new_subscriber).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            println!("Failed to execute query: {}.", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
