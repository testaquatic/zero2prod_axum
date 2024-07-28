use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::TestApp;

trait TestAppNewletterExt {
    async fn create_unconfirmed_subscriber(self) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

impl TestAppNewletterExt for TestApp {
    /// 테스트 대상 애플리케이션의 퍼블릭 API를 사용해서 확인되지 않은 구독자를 생성한다.
    async fn create_unconfirmed_subscriber(self) -> Result<Self, anyhow::Error> {
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let _mock_guard = Mock::given(path("/email"))
            .and(method(http::Method::POST))
            .respond_with(ResponseTemplate::new(http::StatusCode::OK))
            .named("Create unconfirmed subscriber.")
            .expect(1)
            .mount_as_scoped(&self.email_mock_server)
            .await;
        self.post_subscriptions(body).await?.error_for_status()?;

        Ok(self)
    }
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscriber() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app()
        .await?
        .create_unconfirmed_subscriber()
        .await?;

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
    let response = reqwest::Client::new()
        .post(test_app.get_uri()?.join("newsletters")?)
        .json(&newsletter_request_body)
        .send()
        .await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::OK);
    Ok(())
    // mock은 Drop, 즉 뉴스레터 이메일을 보내지 않았음을 검증한다.
}
