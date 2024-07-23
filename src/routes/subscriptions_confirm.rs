use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};
use http::StatusCode;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn confirm(_parameters: Query<Parameters>) -> Response {
    StatusCode::OK.into_response()
}
