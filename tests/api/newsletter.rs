use reqwest::{Method, StatusCode};
use secrecy::ExposeSecret;
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
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await
    });

    let response = app
        .post_newsletters(&newsletter_request_body, Some(&app.auth_token))
        .await;

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
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await
    });
    let response = app
        .post_newsletters(&newsletter_request_body, Some(&app.auth_token))
        .await;

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
                "text_content": "Newsletter body as plain text",
                "html_content": "<p>Newsletter body as HTML</p>",
                "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await,
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter!",
                "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await,
            }),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app
            .post_newsletters(&invalid_body, Some(&app.auth_token))
            .await;

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
    let body = serde_json::json!({
        "name": "le guin",
        "email": "ursula_le_guin@gmail.com",
    });

    let _mock_guard = Mock::given(matchers::path("/email"))
        .and(matchers::method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(&body).await.error_for_status()?;

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
async fn request_missing_auth_token_are_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    let response = app
        .post_newsletters(
            &serde_json::json!({
                "title": "Newsletter title",
                "text_content": "Newsletter body as plain text",
                "html_content": "<p>Newsletter body as HTML</p>",
                "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await,
            }),
            None,
        )
        .await;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );

    Ok(())
}

#[tokio::test]
async fn invalid_token_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let token = app.invalid_token().await;

    let response = app
        .api_client
        .post(&format!("{}/admin/newsletters", app.app_address()))
        .bearer_auth(token.expose_secret())
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as HTML</p>",
            "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await,
        }))
        .send()
        .await?;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );

    Ok(())
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() -> Result<(), anyhow::Error> {
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
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": app.get_idempotency_key(Some(&app.auth_token)).await,
    });

    // 첫번째 발행 요청
    let response = app
        .post_newsletters(&newsletter_request_body, Some(&app.auth_token))
        .await;
    // 요청이 성공했으므로 이메일이 전송되었다.
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );

    // 두번째 발행 요청
    let response = app
        .post_newsletters(&newsletter_request_body, Some(&app.auth_token))
        .await;
    // 요청이 성공했으므로 이메일이 전송되었다.
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );

    // 이메일이 두번 전송되었으므로 실패

    Ok(())
}
