//! 쿠키를 쉽게 관리하기 위한 모듈이다.

use axum::{extract::FromRequestParts, http::StatusCode, response::IntoResponseParts};

use cookie::Cookie;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha3::Sha3_256;
use tower_cookies::Cookies;

/// 관리하는 쿠키를 표시한다.
#[derive(Default, serde::Deserialize, Clone)]
pub struct CookieFeeder {
    /// 클라이언트에 전달하는 메시지
    pub message: Option<String>,
    /// 클라이언트에 전달하는 사용자 이름
    pub username: Option<String>,
    /// 검증을 위한 hmac
    hmac: Option<String>,
    /// 쿠키 상자
    #[serde(skip)]
    cookies: Option<Cookies>,
}

/// hmac에 사용하는 비밀번호
#[derive(Clone)]
pub struct HmacSecret(pub SecretString);

impl CookieFeeder {
    /// 쿠키 이름
    pub const MESSAGE: &'static str = "_cookie_feeder_messsage";
    pub const USERNAME: &'static str = "_cookie_feeder_username";
    pub const HMAC: &'static str = "_cookie_feeder_hmac";

    /// 쿠키를 생성한다.
    pub fn new(
        message: Option<String>,
        username: Option<String>,
        hmac: Option<String>,
        cookies: Option<Cookies>,
    ) -> CookieFeeder {
        CookieFeeder {
            message,
            username,
            hmac,
            cookies,
        }
    }

    /// hmac과 쿠키의 정보가 일치하는지 확인한다.
    pub fn check_hmac(&self, secret: &HmacSecret) -> Result<bool, anyhow::Error> {
        let hmac = self.generate_hmac(secret)?;
        Ok(Some(hmac) == self.hmac)
    }

    /// message와 username을 사용해서 hmac 문자열을 생성한다.
    pub fn generate_hmac(&self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let hmac = generate_hmac(secret, self.message.as_deref(), self.username.as_deref())?;
        Ok(hmac)
    }

    /// 유효한 hmac을 설정한다.
    pub fn set_hmac(
        secret: &HmacSecret,
        message: Option<String>,
        username: Option<String>,
        cookies: Option<Cookies>,
    ) -> Result<CookieFeeder, anyhow::Error> {
        let hmac = generate_hmac(secret, message.as_deref(), username.as_deref())?;
        Ok(CookieFeeder::new(message, username, Some(hmac), cookies))
    }

    /// hmac 문자열에 접근하는 뷰이다.
    pub fn hmac_ref(&self) -> Option<&str> {
        self.hmac.as_deref()
    }
}

impl<S> FromRequestParts<S> for CookieFeeder
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let hmac = cookies
            .get(CookieFeeder::HMAC)
            .map(|cookie| cookie.value().to_owned());
        let message = cookies
            .get(CookieFeeder::MESSAGE)
            .map(|cookie| cookie.value().to_owned());
        let username = cookies
            .get(CookieFeeder::USERNAME)
            .map(|cookie| cookie.value().to_owned());
        if message.is_none() && username.is_none() {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }

        Ok(Self {
            message,
            username,
            hmac,
            cookies: Some(cookies),
        })
    }
}

impl IntoResponseParts for CookieFeeder {
    type Error = StatusCode;
    fn into_response_parts(
        self,
        res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        let Some(cookies) = self.cookies else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        if let Some(hmac) = self.hmac {
            cookies.add(Cookie::new(CookieFeeder::HMAC, hmac));
        } else {
            cookies.remove(Cookie::from(CookieFeeder::HMAC));
        }

        if let Some(message) = self.message {
            cookies.add(Cookie::new(CookieFeeder::MESSAGE, message));
        } else {
            cookies.remove(Cookie::from(CookieFeeder::MESSAGE));
        }

        if let Some(username) = self.username {
            cookies.add(Cookie::new(CookieFeeder::USERNAME, username));
        } else {
            cookies.remove(Cookie::from(CookieFeeder::USERNAME));
        }

        Ok(res)
    }
}

/// hmac을 생성한다.
fn generate_hmac(
    secret: &HmacSecret,
    message: Option<&str>,
    username: Option<&str>,
) -> Result<String, hmac::digest::InvalidLength> {
    let mut hmac = Hmac::<Sha3_256>::new_from_slice(secret.0.expose_secret().as_bytes())?;
    if let Some(message) = message {
        hmac.update(message.as_bytes());
    }
    if let Some(username) = username {
        hmac.update(username.as_bytes());
    }

    Ok(format!("{:x}", hmac.finalize().into_bytes()))
}
