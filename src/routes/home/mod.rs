use axum::response::IntoResponse;

pub async fn home() -> impl IntoResponse {
    (
        http::StatusCode::OK,
        [(http::header::CONTENT_TYPE, "text/html")],
        include_str!("home.html"),
    )
}