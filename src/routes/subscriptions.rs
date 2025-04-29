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
};

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

/// /subscriptions 핸들러이다.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(pool: State<ZPgPool>, form: Form<FormData>) -> Response {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
