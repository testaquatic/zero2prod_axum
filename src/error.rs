use serde_json::error;

#[derive(thiserror::Error)]
pub enum Zero2ProdAxumError {
    #[error(transparent)]
    EmailClientError(#[from] EmailClientError),
    #[error(transparent)]
    DomainError(#[from] DomainError),
}

#[derive(thiserror::Error, Debug)]
pub enum EmailClientError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("A error is occured: {0}")]
    SubscriberEmailError(String),
    #[error("A error is occured: {0}")]
    SubscriberNameError(String),
}

impl std::fmt::Debug for Zero2ProdAxumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        dive_into_error_to_source(self, f)
    }
}

fn dive_into_error_to_source(
    mut error: &dyn std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> Result<(), std::fmt::Error> {
    std::fmt::Debug::fmt(error, f)?;
    while let Some(e) = error.source() {
        writeln!(f, "\n    caused by:")?;
        std::fmt::Debug::fmt(e, f)?;
        error = e;
    }

    Ok(())
}
