use axum::response::{self, IntoResponse};
use axum_flash::IncomingFlashes;

// 가공되지 않은 요청에 더 이상 접근하지 않아도 된다.
pub async fn login_form(flashes: IncomingFlashes) -> impl IntoResponse {
    let error_html = flashes
        .iter()
        // 오류뿐 아니라 모든 레벨의 메시지를 표시한다.
        .map(|(_, e)| format!("<p><i>{}</i></p>", e))
        .collect::<String>();

    (
        // 더 이상 쿠키를 제거하지 않아도 된다.
        flashes,
        response::Html(format!(include_str!("login.html"), error_html = error_html)),
    )
}
