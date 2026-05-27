use secrecy::SecretString;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UsernamePassword {
    pub username: String,
    #[schema(value_type = String)]
    pub password: SecretString,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    #[schema(value_type = String)]
    pub token: String,
    pub token_type: String,
}
