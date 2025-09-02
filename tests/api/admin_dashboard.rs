use crate::helpers::{assert_is_redirect_to, spawn_app};
use reqwest::StatusCode;

/// 관리자 패널에 접근하려면 로그인을 해야한다.
#[tokio::test]
async fn you_must_logged_in_to_access_the_admin_dashboard() {
    // 준비
    let app = spawn_app().await;

    // 실행
    let response = app.get_admin_dashboard().await;

    // 검증
    assert_is_redirect_to(&response, "/login");
}

/// 로그인에 성공하면 username 쿠키가 설정되어야 한다.
#[tokio::test]
async fn username_cookie_is_set_on_successful_login() {
    // 준비
    let app = spawn_app().await;

    // 실행
    let response = app
        .post_login(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password,
        }))
        .await;
    // 리다이렉트를 따라간다.
    assert_is_redirect_to(&response, "/admin/dashboard");
    let reponse = app.get_admin_dashboard().await;

    // 검증
    assert_eq!(reponse.status(), StatusCode::OK);
    assert_eq!(
        app.get_manage_cookies("/admin")
            .await
            .unwrap()
            .username
            .unwrap(),
        app.test_user.username
    );
}
