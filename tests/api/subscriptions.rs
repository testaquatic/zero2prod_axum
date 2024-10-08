use crate::helpers::TestApp;
use anyhow::Context;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod_axum::database::PostgresPool;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), anyhow::Error> {
    // 테스트 데이터
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 준비
    let test_app = TestApp::spawn_app().await?;
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .mount(&test_app.email_mock_server)
        .await;

    // 실행
    let response = test_app.post_subscriptions(body).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() -> Result<(), anyhow::Error> {
    // 테스트 데이터
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 준비
    let test_app = TestApp::spawn_app().await?;
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .mount(&test_app.email_mock_server)
        .await;

    // 실행
    test_app.post_subscriptions(body).await?;

    // 확인
    let pool = PostgresPool::connect(test_app.settings.database.connect_options_with_db())?;
    #[derive(sqlx::FromRow)]
    struct Saved {
        email: String,
        name: String,
        status: String,
    }
    let saved: Saved = sqlx::query_as("SELECT email, name, status FROM subscriptions;")
        .fetch_one(pool.as_ref())
        .await?;
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() -> Result<(), anyhow::Error> {
    // 테스트 데이터
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    // 준비
    let test_app = TestApp::spawn_app().await?;

    for (invalid_body, error_messages) in test_cases {
        // 실행
        let response = test_app.post_subscriptions(invalid_body).await?;

        // 확인
        assert_eq!(
            response.status(),
            http::StatusCode::UNPROCESSABLE_ENTITY,
            // 테스트 실패시 출력할 커스터마이즈된 추가 오류 메세지
            "The API did not fail with 400 Bad Request when the payload was {},",
            error_messages,
        );
    }

    Ok(())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() -> Result<(), anyhow::Error>
{
    // 테스트 데이터
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitelyNotAnEmail", "invalid email"),
    ];

    // 준비
    let app = TestApp::spawn_app().await?;

    for (body, description) in test_cases {
        // 실행
        let response = app.post_subscriptions(body).await?;

        // 확인
        assert_eq!(
            // 더이상 200 OK가 아니다.
            http::StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }

    Ok(())
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() -> Result<(), anyhow::Error> {
    // 테스트 데이터
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 준비
    let test_app = TestApp::spawn_app().await?;
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .expect(1)
        .mount(&test_app.email_mock_server)
        .await;

    // 실행
    test_app.post_subscriptions(body).await?;

    Ok(())
    // 확인
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_link() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // 여기에서는 더 이상 기댓값을 설정하지 않는다.
    // 테스트는 앱 동작의 다른 측면에 집중한다.
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .mount(&test_app.email_mock_server)
        .await;

    // 실행
    test_app.post_subscriptions(body).await?;

    // 확인
    let email_request = &test_app
        .email_mock_server
        .received_requests()
        .await
        .context("No received request")?[0];
    let confirmation_links = test_app.get_confirmation_links(email_request)?;

    // 두 링크는 동일해야 한다.
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    Ok(())
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 데이터베이스를 무시한다.
    let pool = PostgresPool::connect(test_app.settings.database.connect_options_with_db())?;
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email;")
        .execute(pool.as_ref())
        .await?;

    // 실행
    let response = test_app.post_subscriptions(body).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);

    Ok(())
}
