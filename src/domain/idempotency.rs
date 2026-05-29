use anyhow::Context;
use axum::{
    body::Body,
    http::{self, HeaderMap, HeaderName, HeaderValue},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug)]
pub struct IdempotencyKey(pub Uuid);

impl AsRef<Uuid> for IdempotencyKey {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

pub struct SavedIdempotencyResponse {
    pub response_status_code: i16,
    pub response_headers: Vec<(String, Vec<u8>)>,
    pub response_body: Vec<u8>,
}

impl IntoResponse for SavedIdempotencyResponse {
    fn into_response(self) -> axum::response::Response {
        let status_code = match http::StatusCode::from_u16(self.response_status_code as u16)
            .with_context(|| format!("Unknown status code: {}", self.response_status_code))
        {
            Ok(status_code) => status_code,
            Err(e) => return AppError::UnexpectedError(e).into_response(),
        };

        let headers = self
            .response_headers
            .into_iter()
            .map(|(name, value)| {
                let name =
                    HeaderName::from_bytes(name.as_bytes()).context("Invalid header name")?;
                let value = HeaderValue::from_bytes(&value).context("Invalid header value")?;
                Ok((name, value))
            })
            .collect::<Result<HeaderMap, anyhow::Error>>();
        let headers = match headers {
            Ok(headers) => headers,
            Err(e) => return AppError::UnexpectedError(e).into_response(),
        };

        let body = Body::from(self.response_body);

        (status_code, headers, body).into_response()
    }
}
