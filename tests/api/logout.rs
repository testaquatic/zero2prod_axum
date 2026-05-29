use reqwest::StatusCode;

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn logout_works() {
    let app = spawn_app().await;
    let response = app.post_logout(&app.auth_token).await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let response = app.get_admin_dashboard(Some(&app.auth_token)).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
