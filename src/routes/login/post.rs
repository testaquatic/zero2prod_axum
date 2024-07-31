use axum::response::IntoResponse;

pub async fn login() -> impl IntoResponse {
    (
        http::StatusCode::SEE_OTHER,
        [(http::header::LOCATION, "/")],
        (),
    )
}
