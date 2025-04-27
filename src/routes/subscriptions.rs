use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    database::{ZDabaBase, ZPgPool},
    domain::{NewSubscriber, SubscriberName},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
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
    let name = match SubscriberName::try_from(form.0.name) {
        Ok(name) => name,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name,
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
            &new_subscriber.email,
            new_subscriber.name.as_ref(),
            &Utc::now(),
        )
        .await
}
