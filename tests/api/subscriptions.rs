use crate::{helpers::spawn_app, subscriptions_confirm::SubscriptionSaved};
use entities::prelude::*;
use migration::Table;
use reqwest::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, QuerySelect, StatementBuilder};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

/// /subscriptions에 유효한 POST요청을 보내면 200 OK를 반환해야 한다.
/// 요청의 Content-Type은 application/x-www-form-urlencoded이어야 한다.
/// 요청 바디에는 name과 email 필드가 있어야 한다.
/// 데이터베이스에 구독 정보가 저장되어야 한다.
/// 구독 정보는 name과 email 필드를 포함해야 한다.
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // 실행
    let response = app.post_subscriptions(body.into()).await;

    // 확인
    // 응답 상태 코드가 200 OK인지 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

/// 데이터베이스에 사용자 이름과 이메일, 인증 여부를 저장하는지 확인한다.
/// 확인 링크에 접속하지 않았다면 "pending_confirmation"이다.
#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // 실행
    app.post_subscriptions(body.into()).await;

    // 확인
    let saved = Subscriptions::find()
        .one(&app.db_pool)
        .await
        .expect("Failed to fetch subscription.");
    // 데이터베이스에 저장된 구독 정보가 올바른지 확인
    assert!(
        saved.is_some(),
        "No subscription was saved to the database."
    );
    assert_eq!(
        saved.as_ref().unwrap().name,
        "le guin",
        "The saved name does not match the expected value."
    );
    assert_eq!(
        saved.as_ref().unwrap().email,
        "ursula_le_guin@gmail.com",
        "The saved email does not match the expected value."
    );
    assert_eq!(
        saved.as_ref().unwrap().status,
        "pending_confirmation",
        "The saved status does not match the expected value."
    );
}

/// /subscriptions에 name이나 email 필드가 없는 POST요청을 보내면 400 Bad Request를 반환해야 한다.
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // 준비
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 실행
        let response = app.post_subscriptions(invalid_body.into()).await;
        // 확인
        // actix-web하고 다르게 axum은 422 Unprocessable Entity를 반환한다.
        assert_eq!(
            response.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not return a 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}

/// 필드의 내용이 유효하지 않으면 400 Bad Request를 반환한다.
#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // 준비
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitly-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // 실행
        let response = app.post_subscriptions(body.into()).await;

        // 확인
        // 필드의 내용이 유효하지 않으면 400 Bad Request를 반환해야 한다.
        assert_eq!(
            response.status(),
            reqwest::StatusCode::BAD_REQUEST,
            "The API did not return a 400 Bad Request when the payload was {description}."
        );
    }
}

/// 가입자 이메일 유효 확인 이메일을 확인하다.(뭔가 음운이 느껴진다!)
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_with_a_link() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .mount(&app.email_server)
        .await;

    // 실행
    app.post_subscriptions(body.into()).await;

    // 확인
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // 동일한 링크가 전송되어야 한다.
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    // Mock 종료
}

/// 이메일 주소가 중복됐을 때 확인 이메일을 다시 보낸다.
#[tokio::test]
async fn subscribe_sends_two_confirmation_emails_if_enter_the_same_email_address() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // 실행
    app.post_subscriptions(body.into()).await;
    app.post_subscriptions(body.into()).await;

    // 확인
    let email_requests = &app.email_server.received_requests().await.unwrap();
    assert_eq!(email_requests.len(), 2);
    let first_confirmation_link = app.get_confirmation_links(&email_requests[0]);
    let second_confirmation_link = app.get_confirmation_links(&email_requests[1]);

    // 다른 링크가 전송되어야 한다.
    assert_ne!(first_confirmation_link.html, second_confirmation_link.html);
    assert_ne!(
        first_confirmation_link.plain_text,
        second_confirmation_link.plain_text
    );
    assert_eq!(
        second_confirmation_link.html,
        second_confirmation_link.plain_text
    );

    // 실행
    // 두번째 링크로 인증할 수 있어야 한다.
    reqwest::get(second_confirmation_link.html)
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

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // 준비
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 작동에 필요한 필수적인 행을 삭제한다.
    app.db_pool
        .execute(StatementBuilder::build(
            Table::alter()
                .table(entities::prelude::Subscriptions)
                .drop_column(entities::subscriptions::Column::Email),
            &DbBackend::Postgres,
        ))
        .await
        .unwrap();

    // 실행
    let response = app.post_subscriptions(body.into()).await;

    // 확인
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}
