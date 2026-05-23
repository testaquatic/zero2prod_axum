#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SubscribeData {
    pub name: String,
    pub email: String,
}
