use axum::{
    Form,
    response::{ErrorResponse, Response},
};
use secrecy::SecretString;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(Form(form): Form<FormData>) -> Result<Response, ErrorResponse> {
    todo!()
}
