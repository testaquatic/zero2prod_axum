use secrecy::SecretString;

pub struct Credentials {
    pub username: String,
    pub password: SecretString,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize)]
pub struct UsernamePassword {
    pub username: String,
    pub password: SecretString,
}
