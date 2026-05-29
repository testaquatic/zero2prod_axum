use fake::Fake;
use reqwest::StatusCode;
use secrecy::ExposeSecret;
use uuid::Uuid;
use zero2prod_axum::domain::response::TokenResponse;

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

#[tokio::test]
async fn changing_password_works() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // 비밀번호를 변경한다.
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
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );

    // 로그아웃 한다
    let response = app.post_logout(&app.auth_token).await;
    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "expected 202 OK but got: {:?}",
        response
    );

    // 새로운 토큰을 생성한다.
    let response = app
        .post_login(
            &serde_json::json!({"username": &app.test_user.username, "password": new_password}),
        )
        .await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );

    let new_token = response
        .json::<TokenResponse>()
        .await
        .expect("failed to get new token")
        .token
        .expose_secret()
        .to_string();

    // 새로운 토큰으로 관리자 대쉬보드를 불러온다
    let response = app.get_admin_dashboard(Some(&new_token)).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "expected 200 OK but got: {:?}",
        response
    );
    Ok(())
}
