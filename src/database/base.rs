use std::{str::FromStr, usize};

use crate::{
    domain::NewSubscriber,
    idempotency::IdempotencyKey,
    settings::DatabaseSettings,
    utils::{error_chain_fmt, AppError500, SubscriptionToken},
};
use axum::{
    body::to_bytes,
    response::{IntoResponse, Response},
};
use http::{HeaderMap, HeaderName, HeaderValue};

use secrecy::Secret;
use sqlx::{postgres::PgHasArrayType, Database, Pool};
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
}

impl std::fmt::Debug for Z2PADBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(sqlx::FromRow)]
pub struct ConfirmedSubscriber {
    pub email: String,
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

impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}

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

impl TryFrom<Response> for SavedHttpResponse {
    type Error = anyhow::Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let response_status_code = response.status().as_u16() as i16;
        let response_headers = response
            .headers()
            .iter()
            .map(|(name, value)| HeaderPairRecord {
                name: name.to_string(),
                value: value.as_bytes().to_vec(),
            })
            .collect::<Vec<_>>();

        let response_body = tokio::runtime::Runtime::new()?
            .block_on(to_bytes(response.into_body(), usize::MAX))?
            .to_vec();

        Ok(SavedHttpResponse {
            response_status_code,
            response_headers,
            response_body,
        })
    }
}

#[trait_variant::make(Send)]
pub trait Z2PADB: AsRef<Pool<Self::DB>> + TryInto<Pool<Self::DB>> + Sized + Clone {
    type Z2PADBPool: Z2PADB<DB = Self::DB>;
    type DB: Database;
    fn connect(database_settings: &DatabaseSettings) -> Result<Self::Z2PADBPool, Z2PADBError>;

    /// 반환 값은 구독자의 uuid이다.
    async fn insert_subscriber(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> Result<SubscriptionToken, Z2PADBError>;

    async fn confirm_subscriber(
        &self,
        subscriber_id: Uuid,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> Result<Option<Uuid>, Z2PADBError>;

    async fn get_confirmed_subscribers(&self) -> Result<Vec<ConfirmedSubscriber>, Z2PADBError>;

    async fn get_user_id(
        &self,
        username: &str,
        password_hash: Secret<String>,
    ) -> Result<Option<Uuid>, Z2PADBError>;

    async fn get_user_credentials(
        &self,
        username: &str,
    ) -> Result<Option<UserCredential>, Z2PADBError>;

    async fn get_username(&self, user_id: Uuid) -> Result<String, Z2PADBError>;

    async fn change_password(
        &self,
        user_id: Uuid,
        password_hash: Secret<String>,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    async fn get_saved_response(
        &self,
        idempotency_key: &IdempotencyKey,
        user_id: Uuid,
    ) -> Result<Option<SavedHttpResponse>, Z2PADBError>;

    async fn save_response(
        &self,
        idempotency_key: &IdempotencyKey,
        user_id: Uuid,
        http_response: Response,
    ) -> Result<Response, Z2PADBError>;
}
