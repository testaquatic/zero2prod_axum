use anyhow::Context;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod_axum::database::PostgresPool;

use crate::helpers::TestApp;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행
    let response = reqwest::get(test_app.subscriptions_confirm_uri()?).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .mount(&test_app.email_mock_server)
        .await;

    // 구독을 신청한다.
    test_app.post_subscriptions(body).await?;
    // 메일 서버에서 송신한 메일을 확인한다.
    let email_request = &test_app
        .email_mock_server
        .received_requests()
        .await
        .context("No received requests")?[0];
    // 수신한 메일에서 링크룰 추출한다.
    let confirmation_links = test_app.get_confirmation_links(email_request)?;
    // 실행
    let response = reqwest::get(confirmation_links.html).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .mount(&test_app.email_mock_server)
        .await;

    test_app.post_subscriptions(body).await?;
    let email_request = &test_app
        .email_mock_server
        .received_requests()
        .await
        .context("No received requests")?[0];
    let confirmation_links = test_app.get_confirmation_links(email_request)?;

    // 실행
    reqwest::get(confirmation_links.html)
        .await?
        .error_for_status()?;

    // 확인
    let pool = PostgresPool::connect(test_app.settings.database.connect_options_with_db())?;

    #[derive(sqlx::FromRow)]
    struct Subscriber {
        email: String,
        name: String,
        status: String,
    }
    let subscriber = sqlx::query_as!(Subscriber, "SELECT email, name, status FROM subscriptions;")
        .fetch_one(pool.as_ref())
        .await?;

    assert_eq!(subscriber.email, "ursula_le_guin@gmail.com");
    assert_eq!(subscriber.name, "le guin");
    assert_eq!(subscriber.status, "confirmed");

    Ok(())
}
