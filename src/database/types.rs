use crate::utils::{error_chain_fmt, AppError500};
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderName, HeaderValue};

use secrecy::Secret;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

// DB 변경을 쉽게 하기 위한 트레이트
#[derive(thiserror::Error)]
pub enum Z2PADBError {
    #[error("SqlxError: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    IOError(std::io::Error),
    #[error(transparent)]
    AzumCoreError(axum::Error),
    #[error("We expected a saved response, we didn't find kt.")]
    NoSavedResponse,
    #[error("expected: NextAction::StartProcessing, actual: NextAction::ReturnSavedResponse")]
    InvalidNextAction,
    #[error(transparent)]
    ConvertError(anyhow::Error),
}

impl std::fmt::Debug for Z2PADBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub struct UserCredential {
    pub user_id: Uuid,
    pub password_hash: Secret<String>,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
pub struct HeaderPairRecord {
    pub name: String,
    pub value: Vec<u8>,
}

pub struct SavedHttpResponse {
    pub response_status_code: i16,
    pub response_headers: Vec<HeaderPairRecord>,
    pub response_body: Vec<u8>,
}

pub enum NextAction<'a> {
    // 나중에 사용할 트랜잭션을 가지고 있다.
    StartProcessing(Transaction<'a, Postgres>),
    ReturnSavedResponse(SavedHttpResponse),
}
/*
impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}
*/

impl IntoResponse for SavedHttpResponse {
    fn into_response(self) -> Response {
        let status_code = match http::StatusCode::from_u16(self.response_status_code as u16) {
            Err(e) => {
                tracing::error!(error = %e, error_details = ?e);
                return AppError500::new(e).into_response();
            }
            Ok(status_code) => status_code,
        };

        let mut header_map = HeaderMap::new();
        for header in self.response_headers.into_iter() {
            let HeaderPairRecord { name, value } = header;
            let header_name = match HeaderName::from_str(&name) {
                Ok(header_name) => header_name,
                Err(e) => return AppError500::new(e).into_response(),
            };
            let header_value = match HeaderValue::from_bytes(&value) {
                Ok(header_value) => header_value,
                Err(e) => return AppError500::new(e).into_response(),
            };
            header_map.append(header_name, header_value);
        }

        (status_code, header_map, self.response_body).into_response()
    }
}
