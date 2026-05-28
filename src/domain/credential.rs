use secrecy::SecretString;
use uuid::Uuid;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
    pub user_id: uuid::Uuid,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub auth_id: Uuid,
    pub exp: i64,
    pub iat: i64,
}
