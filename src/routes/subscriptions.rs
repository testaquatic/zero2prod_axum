use std::{error::Error, sync::Arc};

use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use rand::{Rng, distr::Alphanumeric, rng};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    database::{
        insert_user_into_database, select_uuid_pending_confirmation_email, store_token_in_database,
        update_token,
    },
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::RouterState,
};

/// /subscriptions 핸들러이다.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    router_state: State<Arc<RouterState>>,
    form: Form<FormData>,
) -> Result<Response, SubscriberError> {
    let new_subscriber = form.0.try_into()?;

    let mut transaction = router_state
        .z_pgpool
        .as_ref()
        .begin()
        .await
        .map_err(SubscriberError::PoolError)?;

    let subscription_token = generate_subscription_token();

    match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => {
            store_token(&mut transaction, &subscriber_id, &subscription_token).await?;
            transaction
                .commit()
                .await
                .map_err(SubscriberError::TransactionCommitError)?;
        }
        Err(e) => {
            let db_e = e.as_database_error();
            match db_e {
                Some(e)
                    if e.is_unique_violation()
                        && e.constraint() == Some("subscriptions_email_key") =>
                {
                    transaction
                        .rollback()
                        .await
                        .map_err(SubscriberError::TransactionCommitError)?;
                    update_token_when_subscribe_twice(
                        router_state.z_pgpool.as_ref(),
                        &new_subscriber.email,
                        &subscription_token,
                    )
                    .await?;
                }
                _ => return Err(StoreTokenError(anyhow::Error::from(e)))?,
            }
        }
    };

    if send_confirmation_email(
        &router_state.email_client,
        new_subscriber,
        &router_state.base_url,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };

    Ok(StatusCode::OK.into_response())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::try_from(value.name)?;
        let email = SubscriberEmail::try_from(value.email)?;
        Ok(Self { email, name })
    }
}

#[tracing::instrument(name = "Update Token in the database", skip_all, err)]
async fn update_token_when_subscribe_twice(
    pgpool: &PgPool,
    subscriber_email: &SubscriberEmail,
    new_token: &str,
) -> Result<Uuid, SubscriberError> {
    let mut transaction = pgpool.begin().await.map_err(SubscriberError::PoolError)?;
    let subscriber_id = match select_uuid_pending_confirmation_email(
        transaction.as_mut(),
        subscriber_email.as_ref(),
    )
    .await
    .map_err(StoreTokenError::from)?
    {
        Some(subscriber_id) => subscriber_id,
        None => {
            return Err(SubscriberError::StoreTokenError(
                sqlx::Error::RowNotFound.into(),
            ));
        }
    };

    while let Err(e) = update_token(transaction.as_mut(), &subscriber_id, new_token).await {
        match e.as_database_error() {
            Some(db_err) => {
                if let Some(code) = db_err.code() {
                    // https://postgresql.kr/docs/10/errcodes-appendix.html 이곳의 에러 코드를 참고했다.
                    if code != "25P02" {
                        transaction = pgpool
                            .begin()
                            .await
                            .map_err(SubscriberError::TransactionCommitError)?;
                    } else {
                        return Err(StoreTokenError(e.into()).into());
                    }
                }
            }
            _ => return Err(StoreTokenError::from(e).into()),
        }
    }

    transaction
        .commit()
        .await
        .map_err(SubscriberError::TransactionCommitError)?;

    Ok(subscriber_id)
}

/// 데이터베이스에 사용자 정보를 저장한다.
#[tracing::instrument(name = "Saving new subscriber details in the database", skip_all, err)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    insert_user_into_database(
        transaction.as_mut(),
        &subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        &Utc::now(),
    )
    .await?;

    Ok(subscriber_id)
}

/// 이메일의 유효성을 확인하는 이메일을 보낸다.
#[tracing::instrument(name = "Send a confirmation email to a new subscriber", skip_all)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await?;

    Ok(())
}

/// 대소문자를 구분하는 무작위 25문자로 구성된 구독 토큰을 생성한다.
fn generate_subscription_token() -> String {
    // API가 변경되었다.
    let rng = rng();
    rng.sample_iter(Alphanumeric)
        .map(|c| c as char)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
    err(Debug),
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    store_token_in_database(transaction.as_mut(), subscriber_id, subscription_token).await?;

    Ok(())
}

pub struct StoreTokenError(anyhow::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token."
        )
    }
}

impl From<sqlx::Error> for StoreTokenError {
    fn from(error: sqlx::Error) -> Self {
        StoreTokenError(anyhow::anyhow!(error))
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }

    Ok(())
}

pub enum SubscriberError {
    ValidationError(String),
    StoreTokenError(StoreTokenError),
    SendEmailError(reqwest::Error),
    PoolError(sqlx::Error),
    InsertSubscriberError(sqlx::Error),
    TransactionCommitError(sqlx::Error),
}

impl From<reqwest::Error> for SubscriberError {
    fn from(e: reqwest::Error) -> Self {
        SubscriberError::SendEmailError(e)
    }
}

impl From<StoreTokenError> for SubscriberError {
    fn from(e: StoreTokenError) -> Self {
        SubscriberError::StoreTokenError(e)
    }
}

impl From<String> for SubscriberError {
    fn from(e: String) -> Self {
        SubscriberError::ValidationError(e)
    }
}

impl std::fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriberError::ValidationError(e) => e.fmt(f),

            SubscriberError::StoreTokenError(_) => write!(
                f,
                "Failed to store the confirmation token for a new subscriber."
            ),
            SubscriberError::SendEmailError(_) => write!(f, "Failed to send a confirmation email."),
            SubscriberError::PoolError(_) => {
                write!(f, "Failed to acquire a Postgres connection from the pool.")
            }
            SubscriberError::InsertSubscriberError(_) => {
                write!(f, "Failed to insert a new subscriber into the database.")
            }
            SubscriberError::TransactionCommitError(_) => write!(
                f,
                "Failed to commit SQL transaction to store a new subscriber."
            ),
        }
    }
}

impl std::fmt::Debug for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for SubscriberError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SubscriberError::ValidationError(_) => None,
            SubscriberError::StoreTokenError(e) => Some(e),
            SubscriberError::SendEmailError(e) => Some(e),
            SubscriberError::PoolError(e) => Some(e),
            SubscriberError::InsertSubscriberError(e) => Some(e),
            SubscriberError::TransactionCommitError(e) => Some(e),
        }
    }
}

impl IntoResponse for SubscriberError {
    fn into_response(self) -> Response {
        tracing::Span::current().record("exception.message", tracing::field::display(&self));
        tracing::Span::current().record("exception.detail", tracing::field::debug(&self));

        match self {
            SubscriberError::ValidationError(_) => StatusCode::BAD_REQUEST.into_response(),
            SubscriberError::StoreTokenError(_)
            | SubscriberError::SendEmailError(_)
            | SubscriberError::PoolError(_)
            | SubscriberError::InsertSubscriberError(_)
            | SubscriberError::TransactionCommitError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
