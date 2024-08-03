use http::HeaderValue;
use uuid::Uuid;

use crate::helpers::TestApp;

#[tokio::test]
async fn you_must_logged_in_to_see_the_change_password_form() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행
    let response = test_app.get_change_password().await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    Ok(())
}

#[tokio::test]
async fn you_must_logged_in_to_change_your_password() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let new_password = Uuid::new_v4().to_string();

    // 실행
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    Ok(())
}

#[tokio::test]
async fn new_password_fields_must_match() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // 실행 - 1단계 - 로그인
    test_app
        .post_login(&serde_json::json!({
            "username": &test_app.test_user.username,
            "password": &test_app.test_user.password
        }))
        .await?;

    // 실행 - 2단계 - 비밀번호 변경을 시도한다.
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": &test_app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &another_new_password
        }))
        .await?;
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/admin/password")?)
    );

    // 실행 - 3단계 - 리다이렉트를 따른다.
    let html_page = test_app.get_change_password_html().await?;
    assert!(html_page.contains("<p><i>새로운 비밀번호가 일치하지 않습니다.</i></p>"));

    Ok(())
}
