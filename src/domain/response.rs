use axum::http;
use secrecy::SecretString;
use uuid::Uuid;

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
pub struct UserInfoResponse {
    #[schema(example = "id")]
    pub username: String,
    #[schema(example = "admin")]
    pub role: String,
}

#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct AppErrorMessageResponse {
    #[serde(serialize_with = "status_code_to_string")]
    #[schema(value_type = String)]
    pub status: http::StatusCode,
    pub message: String,
}

#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct IdempotencyKeyResponse {
    #[schema(example = "7d6f4d8a-4c8d-4d8a-8d6f-4d8a4d8a4d8a", value_type = String)]
    pub idempotency_key: Uuid,
}
