use std::{error::Error, sync::Arc};

use anyhow::Context;
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
    let new_subscriber = form
        .0
        .try_into()
        .map_err(SubscriberError::ValidationError)?;

    let mut transaction = router_state
        .z_pgpool
        .as_ref()
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    let subscription_token = generate_subscription_token();

    match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => {
            store_token(&mut transaction, &subscriber_id, &subscription_token)
                .await
                .context("Failed to store the confirmation token for a new subscriber.")?;
            transaction
                .commit()
                .await
                .context("Failed to commit SQL transaction to store a new subscriber.")?;
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
                        .context("Failed to rollback transaction.")?;
                    update_token_when_subscribe_twice(
                        router_state.z_pgpool.as_ref(),
                        &new_subscriber.email,
                        &subscription_token,
                    )
                    .await?;
                }
                _ => {
                    return Err(anyhow::anyhow!(e).into());
                }
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

async fn update_token_when_subscribe_twice(
    pgpool: &PgPool,
    subscriber_email: &SubscriberEmail,
    new_token: &str,
) -> Result<Uuid, SubscriberError> {
    let mut transaction = pgpool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;
    let subscriber_id = match select_uuid_pending_confirmation_email(
        transaction.as_mut(),
        subscriber_email.as_ref(),
    )
    .await
    .context("Failed to get subscriber_id from the database.")?
    {
        Some(subscriber_id) => subscriber_id,
        None => {
            return Err(anyhow::anyhow!("Failed to find subscriber_id in the database.").into());
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
                            .context("Failed to acquire a Postgres connection from the pool.")?;
                    } else {
                        return Err(
                            anyhow::anyhow!("Failed to update token in the database.").into()
                        );
                    }
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Failed to update token in the database.").into());
            }
        }
    }

    transaction
        .commit()
        .await
        .context("Failed to commit transaction.")?;

    Ok(subscriber_id)
}

/// 데이터베이스에 사용자 정보를 저장한다.
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
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

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

pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    store_token_in_database(transaction.as_mut(), subscriber_id, subscription_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

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

#[derive(thiserror::Error)]
pub enum SubscriberError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for SubscriberError {
    fn into_response(self) -> Response {
        tracing::Span::current().record("exception.message", tracing::field::display(&self));
        tracing::Span::current().record("exception.detail", tracing::field::debug(&self));

        match self {
            SubscriberError::ValidationError(_) => StatusCode::BAD_REQUEST.into_response(),
            SubscriberError::UnexpectedError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
