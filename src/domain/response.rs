use axum::http;
use secrecy::SecretString;

use crate::domain::serializer::{secret_string_to_string, status_code_to_string};

#[derive(Debug, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
pub struct TokenResponse {
    #[serde(serialize_with = "secret_string_to_string")]
    #[schema(value_type = String, example = "token")]
    pub token: SecretString,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct UserInfo {
    #[schema(example = "id")]
    pub username: String,
    #[schema(example = "admin")]
    pub role: String,
}

#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct AppErrorMessage {
    #[serde(serialize_with = "status_code_to_string")]
    #[schema(value_type = String)]
    pub status: http::StatusCode,
    pub message: String,
}
