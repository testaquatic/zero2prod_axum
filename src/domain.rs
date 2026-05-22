#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SubscribeFormData {
    name: String,
    email: String,
}
