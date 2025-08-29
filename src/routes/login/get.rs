use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use cookie::Cookie;

use crate::startup::IndexHtml;

/// GET /login을 담당하는 핸들러
pub async fn login_form(
    State(index_html): State<Arc<IndexHtml>>,
    cookie_jar: CookieJar,
) -> Response {
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
        Body::new(index_html.pub_html.clone()),
    )
        .into_response()
}
