use axum::extract::FromRequestParts;
use tower_sessions::Session;
use uuid::Uuid;

/// [Crate tower_sessions](https://docs.rs/tower-sessions/latest/tower_sessions/) 이 문서를 참고로 했다.
pub struct TypedSession {
    session: Session,
}

impl TypedSession {
    const USER_ID_KEY: &'static str = "userid";

    /// 세션을 회전시킨다.
    pub async fn renew(&self) -> Result<(), tower_sessions::session::Error> {
        self.session.cycle_id().await
    }

    /// 세션에 user_id를 넣는다.
    pub async fn insert_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<(), tower_sessions::session::Error> {
        self.session.insert(Self::USER_ID_KEY, user_id).await
    }

    /// 세션에서 user_id를 추출한다.
    pub async fn get_user_id(&self) -> Result<Option<Uuid>, tower_sessions::session::Error> {
        self.session.get(Self::USER_ID_KEY).await
    }

    /// 세션에 저장된 모든 데이터를 지운다.
    pub async fn flush(&self) -> Result<(), tower_sessions::session::Error> {
        self.session.flush().await
    }
}

/// [Crate tower_sessions](https://docs.rs/tower-sessions/latest/tower_sessions/) 이 문서를 참고로 했다.
impl<S> FromRequestParts<S> for TypedSession
where
    S: Send + Sync,
{
    type Rejection = <Session as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        Ok(Self { session })
    }
}
