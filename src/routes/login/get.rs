use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use tower_cookies::Cookies;

use crate::{cookie::CookieFeeder, startup::IndexHtml};

/// GET /login을 담당하는 핸들러
pub async fn login_form(State(index_html): State<Arc<IndexHtml>>, cookies: Cookies) -> Response {
    (
        // 응답코드
        StatusCode::OK,
        // 헤더
        (
            AppendHeaders([(header::CONTENT_TYPE, "text/html")]),
            // 쿠키를 삭제한다.
            CookieFeeder::new(None, None, None, Some(cookies)),
        ),
        // 응답본문
        Body::new(index_html.pub_html.clone()),
    )
        .into_response()
}
