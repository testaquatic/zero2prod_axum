use anyhow::Context;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{ConfirmationLinks, TestApp};

trait TestAppNewletterExt {
    async fn create_unconfirmed_subscriber(&self) -> Result<ConfirmationLinks, anyhow::Error>;
    async fn create_confirmed_subscriber(&self) -> Result<(), anyhow::Error>;
}

impl TestAppNewletterExt for TestApp {
    /// 테스트 대상 애플리케이션의 퍼블릭 API를 사용해서 확인되지 않은 구독자를 생성한다.
    async fn create_unconfirmed_subscriber(&self) -> Result<ConfirmationLinks, anyhow::Error> {
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let _mock_guard = Mock::given(path("/email"))
            .and(method(http::Method::POST))
            .respond_with(ResponseTemplate::new(http::StatusCode::OK))
            .named("Create unconfirmed subscriber.")
            .expect(1)
            .mount_as_scoped(&self.email_mock_server)
            .await;
        self.post_subscriptions(body).await?.error_for_status()?;
        // mock Postmark 서버가 받은 요청을 확인해서 확인 링크를 추출하고 그것을 반환한다.
        let email_request = &self
            .email_mock_server
            .received_requests()
            .await
            .unwrap()
            .pop()
            .unwrap();

        Ok(self.get_confirmation_links(email_request)?)
    }

    async fn create_confirmed_subscriber(&self) -> Result<(), anyhow::Error> {
        // 동일한 헬퍼를 재사용해서 해당 확인 링크를 실제로 호출하는 단계를 추가한다.
        let confirmation_link = self.create_unconfirmed_subscriber().await?;
        reqwest::get(confirmation_link.html)
            .await
            .unwrap()
            .error_for_status()?;

        Ok(())
    }
}

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
    assert_eq!(response.status(), http::StatusCode::OK);
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
    assert_eq!(response.status(), http::StatusCode::OK);

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

    Ok(())
}
