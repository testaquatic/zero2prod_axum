use argon2::password_hash::rand_core::impls;
use axum::response::{IntoResponse, Response};

pub async fn home() -> impl IntoResponse {
    (
        http::StatusCode::OK,
        [(http::header::CONTENT_TYPE, "text/html")],
        include_str!("home.html"),
    )
}
