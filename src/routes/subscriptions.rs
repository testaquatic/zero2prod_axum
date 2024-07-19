use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use uuid::Uuid;

use crate::{database::basic::Zero2ProdAxumDatabase, settings::DefaultDBPool};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// Form 직렬화 실패 => 422 Unprocessable Entity
// `curl --request POST --data 'email=thomas_mann@hotmail.com&name=Tom' --verbose http://127.0.0.1:8000/subscriptions`
// 데이터베이스 오류 => 500 Internal Server Error
// 정상 작동 => 200 OK
// `curl --request POST --data 'name=le%20guin' --verbose http://127.0.0.1:8000/subscriptions` => 200 OK(처음 실행했을 때) or 500 OK
#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip_all,
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    State(pool): State<Arc<DefaultDBPool>>,
    // axum의 특성상 Form은 마지막으로 가야 한다.
    Form(form): Form<FormData>,
) -> Response {
    // `Result`는 `Ok`와 `Err`라는 두개의 변형을 갖는다.
    // 첫 번째는 성공, 두 번째는 실패를 의미한다.
    // `match` 구문을 사용해서 결과에 따라 무엇을 수행할지 선택한다.
    match pool
        .as_ref()
        .insert_subscriber(&form.email, &form.name)
        .await
    {
        Ok(_) => axum::http::StatusCode::OK.into_response(),
        Err(e) => {
            println!("Failed to execute query: {}.", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
