use anyhow::Context;
use base64::Engine;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use secrecy::Secret;

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

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

pub fn basic_authentication(headers: &http::HeaderMap) -> Result<Credentials, anyhow::Error> {
    // 헤더값이 존재한다면 유효한 UTF8 문자열이어야 한다.
    let header_value = headers
        .get(http::header::AUTHORIZATION)
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The `Authorization` header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    // `:` 구분자를 사용해서 두 개의 세그먼트로 나눈다.
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or(anyhow::anyhow!(
            "A username must be provide in 'Basic' auth."
        ))?
        .to_string();
    let password = credentials
        .next()
        .ok_or(anyhow::anyhow!(
            "A password must be provided in 'Basic' auth."
        ))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
