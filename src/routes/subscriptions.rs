use crate::{database::basic::Zero2ProdAxumDatabase, settings::DefaultDBPool};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use http::StatusCode;
use std::sync::Arc;
// `String`과 `&str`에 `graphemes` 메서드를 제공하기 위한 확장 트레이트
use unicode_segmentation::UnicodeSegmentation;

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
    if is_valid_name(&form.name) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    // `Result`는 `Ok`와 `Err`라는 두개의 변형을 갖는다.
    // 첫 번째는 성공, 두 번째는 실패를 의미한다.
    // `match` 구문을 사용해서 결과에 따라 무엇을 수행할지 선택한다.
    match pool
        .as_ref()
        .insert_subscriber(&form.email, &form.name)
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            println!("Failed to execute query: {}.", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// 입력이 subscriber 이름에 대한 검증 제약 사항을 모두 만족하면 `true`를 반환한다.
/// 그렇지 않으면 `false`를 반환한다.
pub fn is_valid_name(s: &str) -> bool {
    // `trim()`은 입력 `s`에 대해 뒤로 계속되는 공백 문자가 없는 뷰를 반환한다.
    // `is_empty()`는 해당 뷰가 문자를 포함하고 있는지 확인한다.
    let is_empty_or_whitespace = s.trim().is_empty();

    // grapheme은 사용자가 인지할 수 있는 문자로서 유니코드 표준에 의해 정의된다.
    // `grapheme` 입력 `s`안의 graphemes에 대한 이터레이터를 반환한다.
    // `true`는 우리가 확장한 grapheme 정의 셋, 즉 권장되는 정의 셋을 사용하기 원함을 의미한다.
    let is_too_long = s.graphemes(true).count() > 256;

    // 어느 입력 `s`의 모든 문자들에 대해 반복하면서 forbidden 배열 안에 있는 문자 중, 어느 하나와 일치하는 문자가 있는지 확인한다.
    let forbidden_characters = [
        '/', '(', ')', '"', '<', '>', '\\', '{', '}', '$', ';', '%', '&', '|',
    ];
    let conatains_forbidden_characters = s.contains(&forbidden_characters);

    // 어느 한 조건이라도 위반하면 `false`를 반환한다.
    !(is_empty_or_whitespace || is_too_long || conatains_forbidden_characters)
}
