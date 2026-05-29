use anyhow::Context;
use axum::{
    body::{Body, to_bytes},
    http::{self, HeaderMap, HeaderName, HeaderValue},
    response::{IntoResponse, Response},
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
    pub response_headers: Vec<HeaderPairRecord>,
    pub response_body: Vec<u8>,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
pub struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

impl TryFrom<HeaderPairRecord> for (HeaderName, HeaderValue) {
    type Error = anyhow::Error;
    fn try_from(value: HeaderPairRecord) -> Result<Self, Self::Error> {
        Ok((value.name.parse()?, HeaderValue::from_bytes(&value.value)?))
    }
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
            .map(|HeaderPairRecord { name, value }| {
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

impl SavedIdempotencyResponse {
    pub async fn extract_response(
        response: Response,
    ) -> Result<SavedIdempotencyResponse, axum::Error> {
        let response_status_code = response.status().as_u16() as i16;
        let response_headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                let name = name.as_str().to_string();
                let value = value.as_bytes().to_vec();
                HeaderPairRecord { name, value }
            })
            .collect();

        let response_body = to_bytes(response.into_body(), usize::MAX)
            .await
            .map(|b| b.to_vec())?;

        Ok(SavedIdempotencyResponse {
            response_status_code,
            response_headers,
            response_body,
        })
    }
}
