use axum::http;
use secrecy::{ExposeSecret, SecretString};
use serde::Serializer;

pub fn secret_string_to_string<S>(
    password: &SecretString,
    s: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
{
    s.serialize_str(password.expose_secret())
}

pub fn status_code_to_string<S>(
    status_code: &http::StatusCode,
    s: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
{
    s.serialize_str(&status_code.to_string())
}
