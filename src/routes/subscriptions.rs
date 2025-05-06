use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use rand::{Rng, distr::Alphanumeric, rng};
use uuid::Uuid;

use crate::{
    database::ZPgPool,
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
pub async fn subscribe(router_state: State<Arc<RouterState>>, form: Form<FormData>) -> Response {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let subscriber_id = match insert_subscriber(&router_state.z_pgpool, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let subscription_token = generate_subscription_token();
    if store_token(&router_state.z_pgpool, &subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if send_confirmation_email(
        &router_state.email_client,
        new_subscriber,
        &router_state.base_url,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
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

/// 데이터베이스에 사용자 정보를 저장한다.
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(zpg_pool, new_subscriber)
)]
pub async fn insert_subscriber(
    zpg_pool: &ZPgPool,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    zpg_pool
        .add_user(
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
    skip(subscription_token, z_pgpool)
)]
pub async fn store_token(
    z_pgpool: &ZPgPool,
    subscriber_id: &Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    z_pgpool
        .store_token(subscriber_id, subscription_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}
