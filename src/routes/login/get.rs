use axum::response::{self, IntoResponse};
use axum_flash::IncomingFlashes;

// 가공되지 않은 요청에 더 이상 접근하지 않아도 된다.
pub async fn login_form(flashes: IncomingFlashes) -> impl IntoResponse {
    let error_html = flashes
        .iter()
        .filter_map(|(l, e)| {
            if l == axum_flash::Level::Error {
                Some(format!("<p><i>{}</i></p>", e))
            } else {
                None
            }
        })
        .collect::<String>();

    (
        // 더 이상 쿠키를 제거하지 않아도 된다.
        flashes,
        response::Html(format!(include_str!("login.html"), error_html = error_html)),
    )
}
