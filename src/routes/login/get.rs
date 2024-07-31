use axum::response::IntoResponse;

pub async fn login_form() -> impl IntoResponse {
    (
        http::StatusCode::OK,
        [(http::header::CONTENT_TYPE, "text/html")],
        include_str!("login.html"),
    )
}
