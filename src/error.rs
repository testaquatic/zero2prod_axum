use std::error::Error;

#[derive(thiserror::Error)]
pub enum Zero2ProdAxumError {
    #[error("Email Client Error")]
    EmailClientError(#[from] EmailClientError),
    #[error("Domain Error")]
    DomainError(#[from] DomainError),
}

#[derive(thiserror::Error)]
pub enum EmailClientError {
    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Url Parse Error")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(thiserror::Error)]
pub enum DomainError {
    #[error("A error is occured: {0}")]
    SubscriberEmailError(String),
    #[error("A error is occured: {0}")]
    SubscriberNameError(String),
}

impl std::fmt::Debug for Zero2ProdAxumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Caused by : {}", self)?;
        if let Some(e) = self.source() {
            writeln!(f, "\n")?;
            std::fmt::Debug::fmt(e, f)?
        }
        Ok(())
    }
}

impl std::fmt::Debug for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        dive_into_error_to_source(self, f)
    }
}

impl std::fmt::Debug for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        dive_into_error_to_source(self, f)
    }
}

fn dive_into_error_to_source(
    mut error: &dyn std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> Result<(), std::fmt::Error> {
    std::fmt::Display::fmt(error, f)?;
    while let Some(e) = error.source() {
        write!(f, "\n    caused by : ")?;
        std::fmt::Debug::fmt(e, f)?;
        error = e;
    }

    Ok(())
}
