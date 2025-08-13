use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};
use std::sync::Arc;

use axum::{Form, extract::State, http::StatusCode};
use entities::{prelude::Subscriptions, subscriptions};
use rand::distr::{Alphabetic, SampleString};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, InsertResult,
    QueryFilter, QuerySelect, TransactionTrait,
};

/// json형식의 body에서 name과 email 필드를 추출한다.
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
pub async fn subscribe(
    State(pool): State<Arc<DatabaseConnection>>,
    State(email_client): State<Arc<EmailClient>>,
    State(base_url): State<Arc<ApplicationBaseUrl>>,
    Form(form): Form<FormData>,
) -> StatusCode {
    // 잘못된 요청이 들어오면 400 Bad Request를 반환한다.
    let Ok(new_subscriber) = NewSubscriber::try_from(form) else {
        return StatusCode::BAD_REQUEST;
    };

    // 트랜잭션을 시작한다.
    let Ok(transaction) = pool.begin().await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    // 이미 가입한 이메일인지 확인한다.
    let subscriber_id = match check_subscriber_exists(&transaction, &new_subscriber.email).await {
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        Ok(Some(subscriber_id)) => subscriber_id.id,
        Ok(None) => {
            // 데이터베이스에 가입자 정보를 저장한다.
            // 실패하면 500 Internal Server Error를 반환한다.
            let Ok(subscriber_id) = insert_subscriber(&transaction, &new_subscriber).await else {
                // 실패 시 에러 메시지를 출력하고, 500 Internal Server Error를 반환한다.
                return StatusCode::INTERNAL_SERVER_ERROR;
            };
            subscriber_id
        }
    };

    // 데이터베이스에 이메일 인증용 토큰을 저장한다.
    let subscription_token = generate_subscription_token();
    if store_token(&transaction, subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // 일단은 테스트 용으로 무의미한 이메일을 신규 가입자에서 전송한다.
    // 가입자 확인 이메일을 전송하고 실패하면 500 Internal Server Error를 반환한다.
    if send_confirmation_email(
        email_client.as_ref(),
        new_subscriber,
        base_url.0.as_ref(),
        subscription_token.as_ref(),
    )
    .await
    .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // 트랜잭션이 실패하면 500 Internal Server Error를 반환한다.
    transaction
        .commit()
        .await
        .map(|_| StatusCode::OK)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// 데이터베이스에 구독자 정보를 저장한다.
#[tracing::instrument(name = "Saving new subscriber details in the database", skip_all)]
async fn insert_subscriber(
    pool: &DatabaseTransaction,
    new_subscriber: &NewSubscriber,
) -> Result<uuid::Uuid, DbErr> {
    let subscriber_id = uuid::Uuid::new_v4();

    // 폼 데이터에서 name과 email을 추출하고 ActiveModel을 생성한다.
    let new_subscription = subscriptions::ActiveModel {
        id: sea_orm::Set(subscriber_id),
        name: sea_orm::Set(new_subscriber.name.as_ref().into()),
        email: sea_orm::Set(new_subscriber.email.as_ref().into()),
        subscribed_at: sea_orm::Set(chrono::Utc::now().into()),
        status: sea_orm::Set("pending_confirmation".into()),
    };

    // 데이터베이스에 구독자 정보를 저장한다.
    Subscriptions::insert(new_subscription).exec(pool).await?;

    Ok(subscriber_id)
}

/// 가입자 이메일의 유효성을 검증하기 위해서 확인 이메일을 전송한다.
#[tracing::instrument(name = "Sending a confirmation email to a new subscriber", skip_all)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    // 확인 링크를 생성한다.
    let confirmation_link =
        format!("{base_url}/subscriptions/confirm?subscription_token={subscription_token}");
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {confirmation_link} to confirm your subscription.",
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{confirmation_link}\">here</a> to confirm your subscription."
    );

    // 이메일을 전송한다.
    email_client
        .send_email(new_subscriber.email, "Welcome!", &plain_body, &html_body)
        .await
}

///  대소문자를 구분하는 무작위 25문자로 구성된 구독 토큰을 생성한다.
/// [Module distr](https://docs.rs/rand/latest/rand/distr/index.html) 문서를 참고했다.
fn generate_subscription_token() -> String {
    Alphabetic.sample_string(&mut rand::rng(), 25)
}

/// "subscription_tokens"테이블에 "subscriber_id"와 "subscription_token"을 저장한다.
#[tracing::instrument(name = "Store subscription token in the database", skip_all)]
async fn store_token(
    pool: &DatabaseTransaction,
    subscriber_id: uuid::Uuid,
    subscription_token: &str,
) -> Result<InsertResult<entities::subscription_tokens::ActiveModel>, DbErr> {
    let new_subscription_token = entities::subscription_tokens::ActiveModel {
        subscriber_id: sea_orm::Set(subscriber_id),
        subscription_token: sea_orm::Set(subscription_token.into()),
    };

    entities::prelude::SubscriptionTokens::insert(new_subscription_token)
        .exec(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {e:?}");
            e
        })
}

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriberID {
    pub id: uuid::Uuid,
}

/// 이미 존재하는 이메일인지 확인한다.
/// "UPDATE"를 할 때 오류를 검사해도 되겠지만 이 방법이 간단하고 명료하다.
#[tracing::instrument(name = "Checking for existing subscriber", skip_all)]
pub async fn check_subscriber_exists(
    transaction: &DatabaseTransaction,
    subscriber_email: &SubscriberEmail,
) -> Result<Option<SubscriberID>, DbErr> {
    entities::prelude::Subscriptions::find()
        .filter(subscriptions::Column::Email.eq(subscriber_email.as_ref()))
        .select_only()
        .column(subscriptions::Column::Id)
        .into_model::<SubscriberID>()
        .one(transaction)
        .await
}
