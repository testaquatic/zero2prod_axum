use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use sha3::Sha3_256;

use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct FlashMessage {
    pub message: String,
    pub hmac: String,
}

/// hmac을 검증한다.
pub async fn hmac_check(
    State(hmac_secret): State<Arc<HmacSecret>>,
    Json(flash_message): Json<FlashMessage>,
) -> StatusCode {
    let Ok(result) = generate_hmac(&hmac_secret, &flash_message.message) else {
        return StatusCode::FORBIDDEN;
    };
    if result == flash_message.hmac {
        StatusCode::OK
    } else {
        StatusCode::FORBIDDEN
    }
}

/// hmac을 생성한다.
pub fn generate_hmac(
    secret: &HmacSecret,
    message: &str,
) -> Result<String, hmac::digest::InvalidLength> {
    let mut hmac = Hmac::<Sha3_256>::new_from_slice(secret.0.expose_secret().as_bytes())?;
    hmac.update(message.as_bytes());

    Ok(format!("{:x}", hmac.finalize().into_bytes()))
}
