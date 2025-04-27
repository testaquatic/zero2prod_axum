use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use uuid::Uuid;

use crate::database::{ZDabaBase, ZPgPool};

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
    match insert_subscriber(&pool, &form).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// 데이터베이스에 사용자 정보를 저장한다.
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, form)
)]
pub async fn insert_subscriber(pool: &ZPgPool, form: &FormData) -> Result<(), sqlx::Error> {
    pool.add_user(&Uuid::new_v4(), &form.email, &form.name, &Utc::now())
        .await
}
