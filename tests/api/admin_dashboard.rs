use http::HeaderValue;

use crate::helpers::TestApp;

#[tokio::test]
async fn you_mut_be_logged_in_to_access_the_admin_dashboard() -> Result<(), anyhow::Error> {
    // 준비
    let app = TestApp::spawn_app().await?;

    // 실행
    let response = app.get_admin_dashboard().await?;

    // 확인
    assert_eq!(response.status(), http::StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(http::header::LOCATION),
        Some(&HeaderValue::from_str("/login")?)
    );

    Ok(())
}
