use anyhow::Context;
use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscriber() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    test_app.create_unconfirmed_subscriber().await?;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        // Postmark에 대한 요청이 없을을 어서트 한다.
        .expect(0)
        .mount(&test_app.email_mock_server)
        .await;

    // 실행

    // 뉴스레터 페이로드의 스켈레톤 구조
    // 뒤에서 수정한다.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = test_app.post_newsletters(newsletter_request_body).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::ACCEPTED);
    test_app.dispatch_all_pending_emails().await?;
    Ok(())
    // mock은 Drop, 즉 뉴스레터 이메일을 보내지 않았음을 검증한다.
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscriber() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    test_app.create_confirmed_subscriber().await?;

    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .expect(1)
        .mount(&test_app.email_mock_server)
        .await;

    // 실행
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = test_app.post_newsletters(newsletter_request_body).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::ACCEPTED);
    test_app.dispatch_all_pending_emails().await?;

    Ok(())
    // Mock은 뉴스레터 이메일을 보냈다는 Drop을 검증한다.
}

#[tokio::test]
async fn newsletters_returns_422_for_invalid_data() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>",
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter!"
            }),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_newsletters(invalid_body).await?;

        // 확인
        assert_eq!(
            response.status(),
            http::StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not with 400 Bad Request when the payload was {}.",
            error_message
        );
        test_app.dispatch_all_pending_emails().await?;
    }

    Ok(())
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    let response = reqwest::Client::new()
        .post(test_app.newsletters_uri()?)
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .context("Failed to execute request.")?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[http::header::WWW_AUTHENTICATE],
        r#"Basic realm="publish""#
    );
    test_app.dispatch_all_pending_emails().await?;

    Ok(())
}

#[tokio::test]
async fn non_existing_user_is_rejected() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    // 무작위 크리덴셜
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let response = reqwest::Client::new()
        .post(test_app.newsletters_uri()?)
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .context("Failed to execute request.")?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[http::header::WWW_AUTHENTICATE],
        r#"Basic realm="publish""#
    );
    test_app.dispatch_all_pending_emails().await?;

    Ok(())
}

#[tokio::test]
async fn invalid_password_is_rejected() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let username = &test_app.test_user.username;
    // 무작위 비밀번호
    let password = Uuid::new_v4().to_string();
    assert_ne!(test_app.test_user.password, password);

    let response = reqwest::Client::new()
        .post(test_app.newsletters_uri()?)
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>"
            }
        }))
        .send()
        .await
        .context("Failed to execute request.")?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[http::header::WWW_AUTHENTICATE],
        r#"Basic realm="publish""#
    );
    test_app.dispatch_all_pending_emails().await?;

    Ok(())
}
