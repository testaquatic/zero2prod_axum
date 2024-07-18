use axum::{
    response::{IntoResponse, Response},
    Form,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// 올바르지 않은 입력 => 422 Unprocessable Entity
// 올바른 입력 => 200 OK
pub async fn subscribe(Form(_form): Form<FormData>) -> Response {
    axum::http::StatusCode::OK.into_response()
}
