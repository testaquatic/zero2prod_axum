use reqwest::{Method, StatusCode};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

use crate::helpers::{ConfirmationLinks, TestApp, spawn_app};

/// 이메일 주소를 확인하지 않은 사용자에게 뉴스레터를 전달하지 않아야 한다.
#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // 준비
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    // Postmark에 대한 요청이 없어야 한다.
    Mock::given(any())
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // 실행

    // 뉴스레터를 생성한다.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    // 뉴스레터 구독자들한테 이메일을 전송한다.
    let response = app.post_newsletters(newsletter_request_body).await;

    // 검증
    assert_eq!(response.status().as_u16(), StatusCode::OK);
}

/// 이메일 주소를 확인한 사용자에게 뉴스레터를 전달해야 한다.
#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // 준비
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    // 유효한 사용자가 한명 있으니 요청을 하나 전송한다.
    Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // 실행
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = app.post_newsletters(newsletter_request_body).await;

    // 확인
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // 준비
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
            serde_json::json!({
                "title": "Newsletter title"
            }),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        // 확인
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 400 Unprocessable Entity when the payload was {error_message}."
        );
    }
}
/// 테스트 대상 애플리케이션의 퍼블릭 API를 사용해서 확인하지 않은 구독자를 생성한다.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Postmark 서버를 모사한다.
    let _mock_guard = Mock::given(path("/email"))
        .and(method(Method::POST))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
    let email_request = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

/// 이메일 유효성을 확인한 구독자를 생성한다.
async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
