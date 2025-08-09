use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    entities::{self, prelude::Subscriptions, subscriptions},
};
use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, InsertResult};

/// 요청의 body에서 name과 email 필드를 추출한다.
#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

/// /subscriptions - POST 핸들러
/// form 데이터에서 name과 email을 추출한 후에 데이터베이스에 저장한다.
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(pool: State<Arc<DatabaseConnection>>, form: Form<FormData>) -> StatusCode {
    let Ok(new_subscriber) = form.0.try_into() else {
        return StatusCode::BAD_REQUEST;
    };

    insert_subscriber(pool.as_ref(), &new_subscriber)
        .await
        // 쿼리 실행 결과를 확인하고, 성공 시 200 OK를 반환한다.
        .map(|_| StatusCode::OK)
        // 실패 시 에러 메시지를 출력하고, 500 Internal Server Error를 반환한다.
        .unwrap_or_else(|e| {
            tracing::error!("Failed to execute query: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// 데이터베이스에 구독자 정보를 저장한다.
#[tracing::instrument(name = "Saving new subscriber details in the database", skip_all)]
async fn insert_subscriber(
    pool: &DatabaseConnection,
    new_subscriber: &NewSubscriber,
) -> Result<InsertResult<subscriptions::ActiveModel>, DbErr> {
    // 폼 데이터에서 name과 email을 추출하고 ActiveModel을 생성한다.
    let new_subscription = entities::subscriptions::ActiveModel {
        id: sea_orm::Set(uuid::Uuid::new_v4()),
        name: sea_orm::Set(new_subscriber.name.as_ref().into()),
        email: sea_orm::Set(new_subscriber.email.as_ref().into()),
        subscribed_at: sea_orm::Set(chrono::Utc::now().into()),
    };

    // 데이터베이스에 구독자 정보를 저장한다.
    Subscriptions::insert(new_subscription).exec(pool).await
}
