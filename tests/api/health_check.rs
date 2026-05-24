use reqwest::StatusCode;

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", app.app_address()))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(Some(0), response.content_length());
}
