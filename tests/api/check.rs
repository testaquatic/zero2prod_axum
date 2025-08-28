use fake::{Fake, Faker};
use reqwest::StatusCode;

use crate::helpers::spawn_app;

/// 빈 요청에 대해서 422 Unprocessable Entity를 반환한다.
#[tokio::test]
async fn check_hamc_returns_442_if_request_is_empty() {
    // 준비
    let app = spawn_app().await;
    let test_case = serde_json::json!({});

    // 실행
    let response = app.post_check_hmac(test_case).await;

    // 검증
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// 올바른 요청에 대해서 200 Ok를 반환한다.
#[tokio::test]
async fn check_hamc_returns_200_if_request_is_valid() {
    // 준비
    let app = spawn_app().await;

    // 실행
    app.post_login(&serde_json::json!({
        "username": Faker.fake::<String>(),
        "password": Faker.fake::<String>(),
    }))
    .await;
    let (message, hmac) = app.get_flash_cookies().await;
    dbg!(message.as_ref().unwrap().as_str());
    let response = app
        .post_check_hmac(serde_json::json!({
            "message": urlencoding::decode(message.unwrap().as_str()).unwrap(),
            "hmac": hmac.unwrap(),
        }))
        .await;

    // 검증
    assert_eq!(response.status(), StatusCode::OK);
}

/// 해시가 일치하지 않으면 대해서 401 Unauthorized를 반환한다.
#[tokio::test]
async fn check_hamc_returns_403_if_request_is_invalid() {
    // 준비
    let app = spawn_app().await;

    // 실행
    app.post_login(&serde_json::json!({
        "username": Faker.fake::<String>(),
        "password": Faker.fake::<String>(),
    }))
    .await;
    let (message, hmac) = app.get_flash_cookies().await;
    dbg!(message.as_ref().unwrap().as_str());
    let response = app
        .post_check_hmac(serde_json::json!({
            "message": urlencoding::decode("Invalid Message").unwrap(),
            "hmac": hmac.unwrap(),
        }))
        .await;

    // 검증
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
