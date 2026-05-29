use reqwest::StatusCode;
use secrecy::ExposeSecret;

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn you_must_be_logged_in_to_access_admin_dashboard() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let response = app
        .api_client
        .get(&format!("{}/admin/dashboard", app.app_address()))
        .send()
        .await?;
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );
    Ok(())
}

#[tokio::test]
async fn admin_dashboard_with_invalid_token_is_rejected_with_a_401() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let token = app.invalid_token().await;

    let response = app.get_admin_dashboard(Some(&token.expose_secret())).await;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );
    Ok(())
}

#[tokio::test]
async fn admin_dashboard_response_with_expected_json() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard(Some(&app.auth_token)).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_json = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(response_json["username"], app.test_user.username);
    assert_eq!(response_json["role"], "admin");

    Ok(())
}
