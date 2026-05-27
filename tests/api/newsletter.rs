use axum::http::HeaderValue;
use reqwest::{Method, StatusCode, header};
use uuid::Uuid;
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers::{
    startup::spawn_app,
    test_app::{ConfirmationLinks, TestApp},
};

#[tokio::test]
async fn newletters_are_not_delivered_to_unconfirmed_subscribers() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    create_unconfirmed_subscriber(&app).await?;

    Mock::given(matchers::any())
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "content": {
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>"
      }
    });
    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );

    Ok(())
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await?;

    Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "content": {
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>"
      }
    });
    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );

    Ok(())
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    let test_cases = vec![
        (
            serde_json::json!({
              "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
              }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 422 Bad Request when the payload was {}.",
            error_message
        );
    }

    Ok(())
}

/// 테스트 대상 애플리케이션의 퍼블릭 API를 사용해서 확인되지 않은 구독자를 생성한다.
async fn create_unconfirmed_subscriber(app: &TestApp) -> Result<ConfirmationLinks, anyhow::Error> {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()?;

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    let confirmation_links = app.get_confirmation_links(email_request);

    Ok(confirmation_links)
}

async fn create_confirmed_subscriber(app: &TestApp) -> Result<(), anyhow::Error> {
    let confirmation_link = create_unconfirmed_subscriber(app).await?;
    reqwest::get(confirmation_link.html)
        .await?
        .error_for_status()?;

    Ok(())
}

#[tokio::test]
async fn request_missing_authorization_are_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", app.app_address()))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );
    assert_eq!(
        response.headers().get(header::WWW_AUTHENTICATE),
        Some(&HeaderValue::from_str(r#"Basic realm="publish""#)?),
        "expected 'WWW-Authenticate' header with value 'Basic realm=\"publish\"' but got: {:?}",
        response
    );

    Ok(())
}

/// cargo nextest run non_existing_user_is_rejected --release -- --nocapture
/// 아래 테스트와 소요 시간을 비교해보자
#[tokio::test]
async fn non_existing_user_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let start_time = tokio::time::Instant::now();

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", app.app_address()))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await?;

    let duration = tokio::time::Instant::now() - start_time;
    println!("Request took {}ms", duration.as_millis());

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );
    assert_eq!(
        response.headers().get(header::WWW_AUTHENTICATE),
        Some(&HeaderValue::from_str(r#"Basic realm="publish""#)?),
        "expected 'WWW-Authenticate' header with value 'Basic realm=\"publish\"' but got: {:?}",
        response
    );

    Ok(())
}

/// cargo nextest run invalid_password_is_rejected --release -- --nocapture
/// 위 테스트와 소요 시간을 비교해보자
/// 소요 시간이 크게 차이난다면 소요시간 분석 부채널 공격의 우려가 있다
/// 사용자가 존재하는지 확인할 수 있다
#[tokio::test]
async fn invalid_password_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let username = &app.test_user.username;
    let password = Uuid::new_v4().to_string();

    assert_ne!(
        app.test_user.password, password,
        "password must be invalid to test"
    );

    let start_time = tokio::time::Instant::now();

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", app.app_address()))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await?;

    let duration = tokio::time::Instant::now() - start_time;
    println!("Request took {}ms", duration.as_millis());

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );
    assert_eq!(
        response.headers().get(header::WWW_AUTHENTICATE),
        Some(&HeaderValue::from_str(r#"Basic realm="publish""#)?),
        "expected 'WWW-Authenticate' header with value 'Basic realm=\"publish\"' but got: {:?}",
        response
    );

    Ok(())
}
