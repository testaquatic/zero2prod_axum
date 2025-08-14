use reqwest::{Method, StatusCode};
use sea_orm::{EntityTrait, QuerySelect};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::spawn_app;

/// 쿼리 스트링에 subscription_token 파라미터가 없으면 400 Bad Request를 반환한다.
#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // 준비
    let app = spawn_app().await;

    // 실행
    // 쿼리 스트링이 없는 요청이다.
    let response = reqwest::get(&format!("{}/subscriptions/confirm", &app.address))
        .await
        .unwrap();

    // 확인
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// 이메일 유효성 확인 주소에 get 요청을 보내면 200 OK를 반환한다.
#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // 가입 요청을 보낸다.
    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // 실행
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // 확인
    assert_eq!(response.status(), StatusCode::OK);
}

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriptionSaved {
    pub email: String,
    pub name: String,
    pub status: String,
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscription() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // 실행
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // 확인
    // 인증 여부를 확인한다.
    // 이 부분은 코드 중복이 한번 더 반복되면 함수화한다.
    let saved = entities::prelude::Subscriptions::find()
        .select_only()
        .columns([
            entities::subscriptions::Column::Email,
            entities::subscriptions::Column::Name,
            entities::subscriptions::Column::Status,
        ])
        .into_model::<SubscriptionSaved>()
        .one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.")
        .unwrap();

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}
