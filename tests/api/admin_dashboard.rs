use http::HeaderValue;

use crate::helpers::TestApp;

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행
    let response = test_app.get_admin_dashboard().await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    Ok(())
}

#[tokio::test]
async fn logout_clears_session_state() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행 - 단계 1 - 로그인한다.
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password,
    });
    let response = test_app.post_login(&login_body).await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/dashboard")?)
    );

    // 실행 - 단계 2 - 리다이렉트를 따른다.
    let html_page = test_app.get_admin_dashboard_html().await?;
    assert!(html_page.contains(&format!("로그인한 사용자: {}", test_app.test_user.username)));

    // 실행 - 단계 3 - 로그아웃 한다.
    let response = test_app.post_logout().await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    // 실행 - 단계 4 - 리다이렉트를 따른다.
    let html_page = test_app.get_login_html().await?;
    assert!(html_page.contains("<p><i>로그아웃 했습니다.</i></p>"));

    // 실행 - 단계 5 - 관리자 패널을 로딩한다.
    let response = test_app.get_admin_dashboard().await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    Ok(())
}
