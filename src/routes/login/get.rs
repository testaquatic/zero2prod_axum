use axum::response::{self, IntoResponse, Response};
use axum_flash::IncomingFlashes;
use std::fmt::Write;

use crate::utils::AppError500;

// 가공되지 않은 요청에 더 이상 접근하지 않아도 된다.
pub async fn login_form(flashes: IncomingFlashes) -> axum::response::Result<Response> {
    // 오류뿐 아니라 모든 레벨의 메시지를 표시한다.
    let mut error_html = String::new();
    for (_, msg) in flashes.iter() {
        write!(error_html, "<p><i>{}</i></p>", msg).map_err(AppError500::new)?;
    }

    let response = (
        // 더 이상 쿠키를 제거하지 않아도 된다.
        flashes,
        response::Html(format!(include_str!("login.html"), error_html = error_html)),
    )
        .into_response();

    Ok(response)
}
