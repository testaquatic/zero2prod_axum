use secrecy::SecretString;

use crate::domain::serializer::secret_string_to_string;

#[derive(Debug, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
pub struct TokenResponse {
    #[serde(serialize_with = "secret_string_to_string")]
    #[schema(value_type = String)]
    pub token: SecretString,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct UserInfo {
    pub username: String,
    pub role: String,
}
