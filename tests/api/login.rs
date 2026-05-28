use reqwest::StatusCode;
use secrecy::SecretString;
use uuid::Uuid;
use zero2prod_axum::{domain::form_data::LoginFormData, error::AppError};

use crate::helpers::startup::spawn_app;

/// cargo nextest run invalid_username_is_rejected --release -- --nocapture
/// 아래 테스트와 소요 시간을 비교해보자
#[tokio::test]
async fn invalid_username_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();
    let username_password = LoginFormData {
        username,
        password: SecretString::new(password.into()),
    };

    let now = std::time::Instant::now();
    let response = app.post_login(&username_password).await;
    let elapsed = now.elapsed();
    println!("elapsed: {}ms", elapsed.as_millis());

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );

    let resonse_json = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        resonse_json["status"],
        axum::http::StatusCode::UNAUTHORIZED.to_string(),
        "response: {:?}",
        resonse_json
    );
    assert_eq!(
        resonse_json["message"],
        AppError::AuthError(anyhow::anyhow!("")).to_string(),
        "response: {:?}",
        resonse_json
    );

    Ok(())
}

/// cargo nextest run invalid_password_is_rejected --release -- --nocapture
/// 위 테스트와 소요 시간을 비교해보자
/// 소요 시간이 크게 차이난다면 소요시간 분석 부채널 공격의 우려가 있다
/// 사용자가 존재하는지 확인할 수 있다
#[tokio::test]
async fn invalid_password_is_rejected() -> Result<(), anyhow::Error> {
    let app = spawn_app().await;
    let username = &app.test_user.username;
    let password = Uuid::new_v4().to_string();
    let username_password = LoginFormData {
        username: username.clone(),
        password: SecretString::new(password.clone().into()),
    };

    assert_ne!(
        app.test_user.password, password,
        "password must be invalid to test"
    );

    let now = std::time::Instant::now();
    let response = app.post_login(&username_password).await;
    let elapsed = now.elapsed();
    println!("elapsed: {}ms", elapsed.as_millis());

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "expected 401 Unauthorized but got: {:?}",
        response
    );

    let resonse_json = response.json::<serde_json::Value>().await.unwrap();
    assert_eq!(
        resonse_json["status"],
        axum::http::StatusCode::UNAUTHORIZED.to_string(),
        "response: {:?}",
        resonse_json
    );
    assert_eq!(
        resonse_json["message"],
        AppError::AuthError(anyhow::anyhow!("")).to_string(),
        "response: {:?}",
        resonse_json
    );

    Ok(())
}
