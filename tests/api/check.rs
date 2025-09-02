use fake::{Fake, Faker};
use reqwest::StatusCode;

use crate::helpers::spawn_app;

/// 빈 요청에 대해서 422 Unprocessable Entity를 반환한다.
#[tokio::test]
async fn check_hamc_returns_403_if_request_is_empty() {
    // 준비
    let app = spawn_app().await;
    let test_case = serde_json::json!({});

    // 실행
    let response = app.post_check_cookie(test_case).await;

    // 검증
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
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
    let cookie_feeder = app.get_manage_cookies("/").await.unwrap();
    let response = app
        .post_check_cookie(serde_json::json!({
            "message": cookie_feeder.message.as_ref().unwrap(),
            "hmac": cookie_feeder.hmac_ref(),
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
    let manage_cookie = app.get_manage_cookies("/").await.unwrap();
    let response = app
        .post_check_cookie(serde_json::json!({
            "message": urlencoding::decode("Invalid Message").unwrap(),
            "hmac":manage_cookie.hmac_ref(),
        }))
        .await;

    // 검증
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
