use crate::entities::{self, prelude::Subscriptions};
use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};

/// 요청의 body에서 name과 email 필드를 추출한다.
#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

/// /subscriptions - POST 핸들러
/// form 데이터에서 name과 email을 추출한 후에 데이터베이스에 저장한다.
pub async fn subscribe(pool: State<Arc<DatabaseConnection>>, form: Form<FormData>) -> StatusCode {
    let request_id = uuid::Uuid::new_v4();
    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );
    log::info!("request_id {request_id} - Saving new subscriber details in the database.");
    // 폼 데이터에서 name과 email을 추출하고 ActiveModel을 생성한다.
    let new_subscription = entities::subscriptions::ActiveModel {
        id: sea_orm::Set(uuid::Uuid::new_v4()),
        name: sea_orm::Set(form.name.clone()),
        email: sea_orm::Set(form.email.clone()),
        subscribed_at: sea_orm::Set(chrono::Utc::now().into()),
    };

    // 데이터베이스에 구독자 정보를 저장한다.
    Subscriptions::insert(new_subscription)
        .exec(pool.as_ref())
        .await
        // 쿼리 실행 결과를 확인하고, 성공 시 200 OK를 반환한다.
        .map(|_| {
            log::info!("request_id {request_id} - New subscriber details have been saved");
            StatusCode::OK
        })
        // 실패 시 에러 메시지를 출력하고, 500 Internal Server Error를 반환한다.
        .unwrap_or_else(|e| {
            log::error!("request_id {request_id} - Failed to execute query: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
