use axum::{async_trait, extract::FromRequestParts};
use http::{request::Parts, StatusCode};
use tower_sessions::Session;
use uuid::Uuid;

pub struct TypedSession {
    session: Session,
}

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";
    pub async fn cycle_id(&self) -> Result<(), tower_sessions::session::Error> {
        self.session.cycle_id().await
    }

    pub async fn insert_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<(), tower_sessions::session::Error> {
        self.session.insert(Self::USER_ID_KEY, user_id).await
    }

    pub async fn get_user_id(&self) -> Result<Option<Uuid>, tower_sessions::session::Error> {
        self.session.get(Self::USER_ID_KEY).await
    }

    pub async fn log_out(&self) -> Result<(), tower_sessions::session::Error> {
        self.session.flush().await
    }
}

// https://docs.rs/axum/0.7.5/axum/extract/index.html#accessing-other-extractors-in-fromrequest-or-fromrequestparts-implementations
// 이 곳의 코드를 참고로 했다.
#[async_trait]
impl<S> FromRequestParts<S> for TypedSession
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(TypedSession {
            session: Session::from_request_parts(req, state).await?,
        })
    }
}
