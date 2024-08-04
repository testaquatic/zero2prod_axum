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
        }))
        .await?;

    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/newsletters")?)
    );

    // 확인
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(html.contains("<p><i>내용을 모두 입력해야 합니다.</i></p>"));
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(!html.contains("<p><i>내용을 모두 입력해야 합니다.</i></p>"));

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
    assert!(html.contains("<p><i>내용을 모두 입력해야 합니다.</i></p>"));
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(!html.contains("<p><i>내용을 모두 입력해야 합니다.</i></p>"));

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
        "html_content": "<p>Newsletter body as HTML</p>"

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
    assert!(html.contains("<p><i>이메일 전송을 완료했습니다.</i></p>"));
    let html = test_app.get_admin_newsletters_html().await?;
    assert!(!html.contains("<p><i>이메일 전송을 완료했습니다.</i></p>"));

    Ok(())
    // Mock은 뉴스레터 이메일을 보냈다는 Drop을 검증한다.
}
