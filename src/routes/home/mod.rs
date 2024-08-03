use axum::response::{self, IntoResponse};

pub async fn home() -> impl IntoResponse {
    response::Html(include_str!("home.html"))
}
