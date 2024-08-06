use axum::response::{IntoResponse, Response};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::task::JoinHandle;

pub struct SubscriptionToken {
    subscription_token: String,
}

impl SubscriptionToken {
    /// 대소문자를 구분하는 무작위 25문자로 구성된 구독 토큰을 생성한다.
    pub fn generate_subscription_token() -> SubscriptionToken {
        let rng = thread_rng();

        let subscription_token = rng
            .sample_iter(Alphanumeric)
            .map(char::from)
            .take(25)
            .collect();

        SubscriptionToken { subscription_token }
    }
}
impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        &self.subscription_token
    }
}

pub fn error_chain_fmt(
    e: &dyn std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> Result<(), std::fmt::Error> {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Casued by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct AppError500(anyhow::Error);

impl std::fmt::Debug for AppError500 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for AppError500 {
    fn into_response(self) -> Response {
        tracing::Span::current()
            .record("error", tracing::field::display(&self))
            .record("error_detail", tracing::field::debug(self));

        http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl AppError500 {
    pub fn new(error: impl Into<anyhow::Error>) -> Self {
        AppError500(error.into())
    }
}

#[derive(thiserror::Error)]
#[error(transparent)]
pub struct AppError400(anyhow::Error);

impl std::fmt::Debug for AppError400 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for AppError400 {
    fn into_response(self) -> Response {
        tracing::Span::current()
            .record("error", tracing::field::display(&self))
            .record("error_detail", tracing::field::debug(self));

        http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl AppError400 {
    pub fn new(error: impl Into<anyhow::Error>) -> Self {
        AppError400(error.into())
    }
}

// `spawn_blocking`으로부터 트레이트 바운드와 시그니처를 복사했다.
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    // 이것이 실행된 후 새로운 스레드를 실행한다.
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
