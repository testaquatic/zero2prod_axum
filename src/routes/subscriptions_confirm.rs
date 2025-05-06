use axum::{
    Form,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[tracing::instrument(name = "Confirm a pending subscriber")]
pub async fn confirm(_parameters: Form<Parameters>) -> Response {
    StatusCode::OK.into_response()
}

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}
