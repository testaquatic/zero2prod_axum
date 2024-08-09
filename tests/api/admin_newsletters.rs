use http::HeaderValue;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_newsletters() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행
    let response = test_app.get_admin_newsletters().await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    Ok(())
}

#[tokio::test]
async fn admin_newsletters_page_must_be_shown_to_logged_in_users() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행
    test_app
        .post_login(&serde_json::json!({
            "username": &test_app.test_user.username,
            "password": &test_app.test_user.password
        }))
        .await?;
    let response = test_app.get_admin_dashboard().await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn users_not_fill_formdata_will_get_flash_message() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행 - 단계 1 - 로그인
    test_app.login_and_get_admin_dashboard_html().await?;

    // 실행 - 단계 2 -  폼데이터 제출: 일부 필드의 문자열의 길이가 0
    let response = test_app
        .post_admin_newsletters(&serde_json::json!({
            "title": "",
            "text_content": "Newsletter body as plain text",
            "html_content": "<p>Newsletter body as HTML</p>",
            "idempotency_key": uuid::Uuid::new_v4(),
        }))
        .await?;

    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/newsletters")?)
    );

    // 확인
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(html.contains("<p><i>입력을 잘못했습니다.</i></p>"));
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(!html.contains("<p><i>입력을 잘못했습니다.</i></p>"));

    // 실행 - 단계 2 -  폼데이터 제출: 일부 필드가 누락됨
    let response = test_app
        .post_admin_newsletters(&serde_json::json!({
            "title": "",
            "text_content": "Newsletter body as plain text",
        }))
        .await?;

    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/newsletters")?)
    );

    // 실행 - 단계 3 - 링크를 따른다.
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(html.contains("<p><i>입력을 잘못했습니다.</i></p>"));
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(!html.contains("<p><i>입력을 잘못했습니다.</i></p>"));

    Ok(())
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
    test_app.login_and_get_admin_dashboard_html().await?;

    // 실행 - 단계 1 - 폼을 채운다.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4(),

    });
    let response = test_app
        .post_admin_newsletters(&newsletter_request_body)
        .await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/newsletters")?)
    );

    // 실행 - 단계 2 - 링크를 따른다.
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(html.contains("<p><i>이메일 전송을 예약했습니다.</i></p>"));
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(!html.contains("<p><i>이메일 전송을 예약했습니다.</i></p>"));

    Ok(())
    // Mock은 뉴스레터 이메일을 보냈다는 Drop을 검증한다.
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    test_app.create_confirmed_subscriber().await?;

    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_mock_server)
        .await;
    test_app.login_and_get_admin_dashboard_html().await?;

    // 실행 - 단계 1 - 뉴스레터 폼을 제출한다.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4(),

    });
    let response = test_app
        .post_admin_newsletters(&newsletter_request_body)
        .await?;

    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/newsletters")?)
    );

    // 실행 - 단계 2 - 리다이렉트를 따른다.
    let html_page = test_app.get_admin_newsletters_html().await?;
    assert!(html_page.contains("<p><i>이메일 전송을 예약했습니다.</i></p>"));

    // 실행 - 단계 3 - 뉴스레터 폼을 다시 제출한다.
    let response = test_app
        .post_admin_newsletters(&newsletter_request_body)
        .await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/newsletters")?)
    );

    // 실행 - 단계 4 - 리다이렉트를 따른다.
    let html_page = test_app.get_admin_newsletters_html().await?;
    assert!(html_page.contains("<p><i>이메일 전송을 예약했습니다.</i></p>"));

    Ok(())
    // Mock은 뉴스레터 이메일을 한 번 보냈다는 드롭을 검증한다.
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    test_app.create_confirmed_subscriber().await?;
    test_app.login_and_get_admin_dashboard_html().await?;

    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        // 두 번째 요청이 첫 번째 요청이 완료되기 전에 들어오도록 충분한 지연시간을 설정한다.
        .respond_with(
            ResponseTemplate::new(http::StatusCode::OK)
                .set_delay(std::time::Duration::from_secs(2)),
        )
        .expect(1)
        .mount(&test_app.email_mock_server)
        .await;

    // 실행 - 두 개의 뉴스레터 폼을 동시에 제출한다.
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4(),
    });
    let response1 = test_app.post_admin_newsletters(&newsletter_request_body);
    let response2 = test_app.post_admin_newsletters(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);
    let (response1, response2) = (response1?, response2?);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(response1.text().await?, response2.text().await?);

    Ok(())
    // Mock은 드롭시 이메일을 한 번만 보냈음을 검증한다.
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() -> Result<(), anyhow::Error>
{
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4(),
    });
    // 한 명의 구독자 대신 두 명의 구독자
    test_app.create_confirmed_subscriber().await?;
    test_app.create_confirmed_subscriber().await?;
    test_app.login_and_get_admin_dashboard_html().await?;

    // 실행 - 단계 1 - 뉴스레터 제출 폼
    // 첫번째 구독자에 대한 이메일 전달은 성공한다.
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .up_to_n_times(1)
        .expect(1)
        .mount(&test_app.email_mock_server)
        .await;
    // 두번째 구독자에 대한 이메일 전달은 실패한다.
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(
            http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
        .up_to_n_times(1)
        .expect(1)
        .mount(&test_app.email_mock_server)
        .await;

    let response = test_app
        .post_admin_newsletters(&newsletter_request_body)
        .await?;
    assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);

    // 실행 - 단계 2 - 폼 제출을 재시도한다.
    // 이제 두명의 구독자 모두에게 이메일 전달을 성공한다.
    Mock::given(path("/email"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(http::StatusCode::OK))
        .expect(1)
        .named("Delivery retry")
        .mount(&test_app.email_mock_server)
        .await;
    let response = test_app
        .post_admin_newsletters(&newsletter_request_body)
        .await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);

    Ok(())

    // mock은 중복된 뉴스레터를 발송하지 않았음을 검증한다.
}
