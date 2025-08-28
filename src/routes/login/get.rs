use std::sync::Arc;

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;

/// GET /login을 담당하는 핸들러
pub async fn login_form(State(index_html): State<Arc<String>>, cookie_jar: CookieJar) -> Response {
    let flash_cookie = cookie_jar
        .remove(Cookie::from("_flash"))
        .remove(Cookie::from("_flash_hmac"));

    (
        // 응답코드
        StatusCode::OK,
        // 헤더
        (
            AppendHeaders([(header::CONTENT_TYPE, "text/html")]),
            flash_cookie,
        ),
        // 응답본문
        index_html.to_string(),
    )
        .into_response()
}
