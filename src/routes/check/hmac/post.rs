use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::cookie::{CookieFeeder, HmacSecret};

/// POST /check/hmac 핸들러이다.
/// hmac을 검증한다.
pub async fn hmac_check(
    State(hmac_secret): State<Arc<HmacSecret>>,
    Json(cookie): Json<CookieFeeder>,
) -> StatusCode {
    if cookie.username.is_none() && cookie.message.is_none() {
        return StatusCode::FORBIDDEN;
    }

    cookie
        .check_hmac(&hmac_secret)
        .map(|ok| {
            if ok {
                StatusCode::OK
            } else {
                StatusCode::FORBIDDEN
            }
        })
        .unwrap_or_else(|e| {
            tracing::error!("Failed to check hmac: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
