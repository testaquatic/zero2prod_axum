use reqwest::StatusCode;
use uuid::Uuid;

use crate::helpers::spawn_app;

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
    let flash_cookies = response.cookies().find(|c| c.name() == "_flash").unwrap();
    assert_eq!(flash_cookies.value(), "Authentication failed");

    // 실행 2
    let flash_cookie = app.get_flash_cookie().await.expect("Empty cookie.");
    assert_eq!(flash_cookie, "Authentication failed")
}
