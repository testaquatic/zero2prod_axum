#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: Databasettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct Databasettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SubscribeFormData {
    pub name: String,
    pub email: String,
}
