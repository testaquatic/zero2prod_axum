use fake::Fake;
use reqwest::StatusCode;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::helpers::startup::spawn_app;

#[tokio::test]
async fn you_must_be_logged_in_to_change_password() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let response = app
        .post_change_password(
            &serde_json::json!({
              "current_password": Uuid::new_v4().to_string(),
              "new_password": new_password,
              "new_password_check": new_password
            }),
            app.invalid_token().await.expose_secret(),
        )
        .await;
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );
    Ok(())
}

#[tokio::test]
async fn new_password_fields_must_match() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();
    let response = app
        .post_change_password(
            &serde_json::json!({
              "current_password": &app.test_user.password,
              "new_password": new_password,
              "new_password_check": another_new_password
            }),
            &app.auth_token,
        )
        .await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "expected 400 Bad Request but got: {:?}",
        response
    );
    Ok(())
}

#[tokio::test]
async fn current_password_must_be_valid() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    let response = app
        .post_change_password(
            &serde_json::json!({
              "current_password": &wrong_password,
              "new_password": new_password,
              "new_password_check": new_password
            }),
            &app.auth_token,
        )
        .await;

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 400 Bad Request but got: {:?}",
        response
    );
    Ok(())
}

#[tokio::test]
async fn too_short_password_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    for _ in 0..10 {
        let new_password = (1..=12).fake::<String>();
        let response = app
            .post_change_password(
                &serde_json::json!({
                  "current_password": &app.test_user.password,
                  "new_password": new_password,
                  "new_password_check": new_password
                }),
                &app.auth_token,
            )
            .await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "expected 400 Bad Request but got: {:?}",
            response
        );
    }

    Ok(())
}

#[tokio::test]
async fn too_long_password_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;

    for _ in 0..10 {
        let new_password = (128..256).fake::<String>();
        let response = app
            .post_change_password(
                &serde_json::json!({
                  "current_password": &app.test_user.password,
                  "new_password": new_password,
                  "new_password_check": new_password
                }),
                &app.auth_token,
            )
            .await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "expected 400 Bad Request but got: {:?}",
            response
        );
    }

    Ok(())
}
