use std::sync::Arc;

use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// /subscriptions 핸들러이다.
pub async fn subscribe(pool: State<Arc<PgPool>>, form: Form<FormData>) -> Response {
    let query_reuslt = sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4);",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await;
    match query_reuslt {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
