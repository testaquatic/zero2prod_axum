#[derive(serde::Serialize, Debug, utoipa::ToSchema)]
pub struct UserInfo {
    #[schema(value_type = String)]
    pub user_id: uuid::Uuid,
    pub username: String,
    pub role: String,
}
