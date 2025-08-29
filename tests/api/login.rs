use reqwest::StatusCode;
use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

/// 로그인에 실패했을 때 _flash 쿠키가 제대로 설정되는지 확인한다.
#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // 준비
    let app = spawn_app().await;

    // 실행 - 로그인을 시도한다.
    let login_body = serde_json::json!({
        "username": Uuid::new_v4().to_string(),
        "password": Uuid::new_v4().to_string(),
    });
    let response = app.post_login(&login_body).await;

    // 확인
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let (flash_cookie, _) = app.get_flash_cookies().await;
    assert_eq!(
        flash_cookie.expect("Empty cookie."),
        urlencoding::encode("Authentication failed")
    );

    // 실행2 - 다시 login 페이지를 로드하면 쿠키가 삭제되어야 한다.
    let response = app
        .api_client
        .get(format!("{}/login", app.address))
        .send()
        .await
        .unwrap();

    // 확인
    assert_eq!(response.status(), StatusCode::OK);
    let (flash_cookie, hamc_cookie) = app.get_flash_cookies().await;
    assert!(flash_cookie.is_none());
    assert!(hamc_cookie.is_none());
}

/// 로그인에 성공하면 대시보드로 리다이렉트 한다.
#[tokio::test]
async fn redirect_to_dashboard_after_login_success() {
    // 준비
    let app = spawn_app().await;

    // 실행 1 - 로그인
    let login_body = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");
}
