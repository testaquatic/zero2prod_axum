use std::str::FromStr;

use crate::{
    domain::NewSubscriber,
    idempotency::IdempotencyKey,
    settings::DatabaseSettings,
    utils::{error_chain_fmt, AppError500, SubscriptionToken},
};
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderName, HeaderValue};

use secrecy::Secret;
use sqlx::{Database, Pool, Transaction};
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

pub enum NextAction<'a, T>
where
    T: sqlx::Database,
{
    // 나중에 사용할 트랜잭션을 가지고 있다.
    StartProcessing(Transaction<'a, T>),
    ReturnSavedResponse(SavedHttpResponse),
}
/*
impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}
*/

impl<'a, T> NextAction<'a, T>
where
    T: sqlx::Database,
{
    pub fn try_get_transaction(self) -> Result<Transaction<'a, T>, Z2PADBError> {
        match self {
            NextAction::ReturnSavedResponse(_) => Err(Z2PADBError::InvalidNextAction),
            NextAction::StartProcessing(transaction) => Ok(transaction),
        }
    }

    pub fn try_get_transaction_mut_ref(&mut self) -> Result<&mut Transaction<'a, T>, Z2PADBError> {
        match self {
            NextAction::ReturnSavedResponse(_) => Err(Z2PADBError::InvalidNextAction),
            NextAction::StartProcessing(transaction) => Ok(transaction),
        }
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
        next_action: NextAction<'_, Self::DB>,
        idempotency_key: &IdempotencyKey,
        user_id: Uuid,
        http_response: Response,
    ) -> Result<Response, Z2PADBError>;

    async fn try_processing(
        &self,
        idempotency_key: &IdempotencyKey,
        user_id: Uuid,
    ) -> Result<NextAction<'_, Self::DB>, Z2PADBError>;

    async fn insert_newsletter_issue(
        next_action: &mut NextAction<'_, Self::DB>,
        title: &str,
        text_content: &str,
        html_content: &str,
    ) -> Result<Uuid, Z2PADBError>;

    async fn enqueue_delivery_tasks(
        next_action: &mut NextAction<'_, Self::DB>,
        newsletter_issue_id: Uuid,
    ) -> Result<<Self::DB as Database>::QueryResult, Z2PADBError>;

    // 함수를 만능으로 만들지 말자
    #[tracing::instrument(name = "Schedule newsletter delivery", skip_all)]
    fn schedule_newsletter_delivery(
        next_action: &mut NextAction<'_, Self::DB>,
        title: &str,
        text_content: &str,
        html_content: &str,
    ) -> impl std::future::Future<Output = Result<(), Z2PADBError>> {
        async {
            let issue_id =
                Self::insert_newsletter_issue(next_action, &title, &text_content, &html_content)
                    .await?;

            Self::enqueue_delivery_tasks(next_action, issue_id).await?;

            Ok(())
        }
    }
}
