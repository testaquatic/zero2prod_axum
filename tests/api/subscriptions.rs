use crate::helpers::{DefaultDBPoolTestExt, TestApp};
use sqlx::FromRow;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod_axum::{database::Zero2ProdAxumDatabase, settings::DefaultDBPool};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() -> Result<(), anyhow::Error> {
    // 테스트 데이터
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // 준비
    let test_app = TestApp::spawn_app().await?;
    let pool = DefaultDBPool::connect(&test_app.settings.database)
        .expect("Failed to connect to Postgres.");

    let mock = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200));

    // 실행
    test_app.test_email_server.test_run(mock).await;
    let response = test_app.post_subscriptions(body).await.unwrap();

    let row = pool
        .fetch_one("SELECT email, name FROM subscriptions;")
        .await
        .expect("Failed to fetch saved subscriptions.");

    // 확인
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    #[derive(sqlx::FromRow)]
    struct Saved {
        email: String,
        name: String,
    }
    let saved = Saved::from_row(&row).expect("Failed to get data from Database.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

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
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
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
            reqwest::StatusCode::BAD_REQUEST,
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
    let mock = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1);

    // 준비
    let test_app = TestApp::spawn_app().await?;
    test_app.test_email_server.test_run(mock).await;

    test_app.post_subscriptions(body).await?;

    Ok(())
}
