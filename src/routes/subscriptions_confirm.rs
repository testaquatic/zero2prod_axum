use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip_all)]
pub async fn confirm(
    State(pool): State<Arc<DatabaseConnection>>,
    Query(parameters): Query<Parameters>,
) -> StatusCode {
    let Ok(id) = get_subscriber_id_from_token(&pool, &parameters.subscription_token).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    match id {
        Some(id) => confirm_subscriber(&pool, id)
            .await
            .map(|_| StatusCode::OK)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        None => StatusCode::UNAUTHORIZED,
    }
}

/// subscriptions테이블에서 id가 subscriber_id인 행의 status 열을 "confirmed"로 변경한다.
#[tracing::instrument(name = "Mark subscriber as confirmed", skip_all)]
pub async fn confirm_subscriber(
    pool: &DatabaseConnection,
    subscriber_id: uuid::Uuid,
) -> Result<(), sea_orm::DbErr> {
    let insert_active_model = entities::subscriptions::ActiveModel {
        id: sea_orm::ActiveValue::Set(subscriber_id),
        status: sea_orm::ActiveValue::Set("confirmed".into()),
        ..Default::default()
    };

    entities::prelude::Subscriptions::update(insert_active_model)
        .filter(entities::subscriptions::Column::Id.eq(subscriber_id))
        .exec(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {e:?}");
            e
        })?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber id from token", skip_all)]
async fn get_subscriber_id_from_token(
    pool: &DatabaseConnection,
    subscription_token: &str,
) -> Result<Option<uuid::Uuid>, sea_orm::DbErr> {
    entities::prelude::SubscriptionTokens::find()
        .column(entities::subscription_tokens::Column::SubscriberId)
        .filter(entities::subscription_tokens::Column::SubscriptionToken.eq(subscription_token))
        .one(pool)
        .await
        .map(|model| model.map(|model| model.subscriber_id))
        .map_err(|e| {
            tracing::error!("Failed to execute query: {e:?}");
            e
        })
}
