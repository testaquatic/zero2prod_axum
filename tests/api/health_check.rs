use reqwest::{StatusCode, header};

use crate::helper::spawn_app;

/// /health_check가 작동하는지 확인한다.
#[tokio::test]
async fn health_check_works() {
    let (_server_handle, address) = spawn_app().await.expect("failed to spawn app");

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let (_server_handle, address) = spawn_app().await.expect("failed to spawn app");

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/subscriptions", address))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let (_server_handle, address) = spawn_app().await.expect("failed to spawn app");

    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", address))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
