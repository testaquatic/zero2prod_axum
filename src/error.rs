use axum::response::IntoResponse;

// 오류 관리를 편하게 하기 위한 래퍼 타입
// 오류 타입을 일치시킬 필요성이 있을 때 사용한다.
#[derive(thiserror::Error)]
pub enum Z2PAError {
    #[error("Email Error: {0}")]
    SubscriberEmailError(String),

    #[error("Name Error: {0}")]
    SubscriberNameError(String),

    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Url Error")]
    UrlParseError(#[from] url::ParseError),

    #[error("IO Error")]
    IOError(#[from] std::io::Error),

    #[error("SQLX Error")]
    SQLXError(#[from] sqlx::Error),

    #[error("Config Error")]
    ConfigError(#[from] config::ConfigError),
}

impl std::fmt::Debug for Z2PAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n    caused by : {}", self)?;
        dive_into_error_to_source(self, f)
    }
}

fn dive_into_error_to_source(
    mut error: &dyn std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> Result<(), std::fmt::Error> {
    loop {
        if let Some(e) = error.source() {
            write!(f, "\n    caused by : ")?;
            std::fmt::Debug::fmt(e, f)?;
            error = e;
        } else {
            return Ok(());
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Z2PAErrResponse {
    #[error(transparent)]
    Zero2ProdAxumErrorResonse(Z2PAError),
    #[error("A database error was encountered while trying to store a subscription token.")]
    StoreTokenError(#[source] sqlx::Error),
}

impl IntoResponse for Z2PAErrResponse {
    fn into_response(self) -> axum::response::Response {
        tracing::Span::current().record("error", self.to_string());
        http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
