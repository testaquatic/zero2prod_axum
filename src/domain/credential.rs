use secrecy::{ExposeSecret, SecretString};
use serde::Serializer;
use uuid::Uuid;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, serde::Serialize)]
pub struct UsernamePassword {
    pub username: String,
    #[schema(value_type = String)]
    #[serde(serialize_with = "secret_string_to_string")]
    pub password: SecretString,
}

fn secret_string_to_string<S>(
    password: &SecretString,
    s: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
{
    s.serialize_str(password.expose_secret())
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, serde::Deserialize)]
pub struct TokenResponse {
    #[serde(serialize_with = "secret_string_to_string")]
    #[schema(value_type = String)]
    pub token: SecretString,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub auth_id: Uuid,
    pub exp: i64,
    pub iat: i64,
}
