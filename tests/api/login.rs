use anyhow::Context;
use http::HeaderValue;

use crate::helpers::TestApp;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() -> Result<(), anyhow::Error> {
    // 준비
    let test_app = TestApp::spawn_app().await?;

    // 실행
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

    let flash_cookies = response
        .cookies()
        .find(|c| c.name() == "_flash")
        .context("No Cookie No Life.")?;
    assert_eq!(
        flash_cookies.value(),
        urlencoding::encode("Authentication failed.")
    );

    // 실행 - 2단계
    let html_page = test_app.get_login_html().await?;
    assert!(html_page.contains(r#"<p><i>Authentication failed.</i></p>"#));

    Ok(())
}
