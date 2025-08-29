use crate::helpers::{assert_is_redirect_to, spawn_app};

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
