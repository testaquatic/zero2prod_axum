use axum::{
    extract::Request,
    middleware::Next,
    response::{self, IntoResponse, Response},
};
use secrecy::Secret;
use tokio::task::JoinHandle;
use tower_sessions::{CachingSessionStore, ExpiredDeletion};
use tower_sessions_moka_store::MokaStore;
use tower_sessions_sqlx_store::PostgresStore;
use uuid::Uuid;

use crate::{database::PostgresPool, session_state::TypedSession, utils::AppError500};

#[derive(Clone)]
pub struct UserId(pub Uuid);

pub async fn reject_anonymous_users(
    typed_session: TypedSession,
    mut request: Request,
    next: Next,
) -> response::Result<Response> {
    match typed_session
        .get_user_id()
        .await
        .map_err(AppError500::new)?
    {
        Some(user_id) => {
            request.extensions_mut().insert(UserId(user_id));
            Ok(next.run(request).await)
        }
        None => {
            tracing::warn!("The user has not logged in");
            Ok(response::Redirect::to("/login").into_response())
        }
    }
}

pub struct PgSessionStorage {
    pub session_store: CachingSessionStore<MokaStore, PostgresStore>,
    pub abort_handle: JoinHandle<Result<(), tower_sessions::session_store::Error>>,
    pub key: Secret<String>,
}

impl PgSessionStorage {
    pub async fn init(
        pool: PostgresPool,
        key: Secret<String>,
    ) -> Result<PgSessionStorage, anyhow::Error> {
        // 세션 저장소를 생성한다.
        let pg_store = PostgresStore::new(pool.into());
        pg_store.migrate().await?;

        // 60초마다 만료된 세션을 삭제한다.
        let deletion_task = tokio::task::spawn(
            pg_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );

        // 확장을 염두에 둔다면 redis가 나은 선택이다.
        // postgres가 백엔드로 작동하고 있으니 작동에 문제는 없을 듯 하다.
        // 세션을 Moka( https://docs.rs/moka/latest/moka/ )로 캐싱한다.
        let caching_store = CachingSessionStore::new(MokaStore::new(Some(5000)), pg_store);
        let session_storage = PgSessionStorage {
            session_store: caching_store,
            abort_handle: deletion_task,
            key,
        };

        Ok(session_storage)
    }
}
