use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
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

    if insert_subscriber(&router_state.z_pgpool, &new_subscriber)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if send_confirmation_email(&router_state.email_client, new_subscriber)
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
) -> Result<(), sqlx::Error> {
    zpg_pool
        .add_user(
            &Uuid::new_v4(),
            new_subscriber.email.as_ref(),
            new_subscriber.name.as_ref(),
            &Utc::now(),
        )
        .await
}

#[tracing::instrument(name = "Send a confirmation email to a new subscriber", skip_all)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://my-api.com/subscriptions/confirm";
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
