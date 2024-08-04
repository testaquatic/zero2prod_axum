use http::HeaderValue;

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
