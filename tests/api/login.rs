use http::HeaderValue;

use crate::helpers::TestApp;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행 - 1단계 - 로그인을 시작한다.
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = test_app.post_login(&login_body).await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    // 쿠키와 관련한 내용은 어서션할 필요가 없다.

    // 실행 - 2단계 - 리다이렉트를 따른다.
    let html_page = test_app.get_login_html().await?;
    assert!(html_page.contains(r#"<p><i>Authentication failed.</i></p>"#));

    // 실행 - 3단계 - 로그인 페이지를 다시 로딩한다.
    let html_page = test_app.get_login_html().await?;
    assert!(!html_page.contains(r#"<p><i>Authentication failed.</i></p>"#));

    Ok(())
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행 - 1단계 - 로그인
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/dashboard")?)
    );

    // 실행 - 2단계 - 리다이렉트를 따른다.
    let html_page = test_app.get_admin_dashboard_html().await?;
    assert!(html_page.contains(&format!("로그인한 사용자: {}", test_app.test_user.username)));

    Ok(())
}
