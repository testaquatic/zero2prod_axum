use secrecy::SecretString;

use crate::domain::serializer::secret_string_to_string;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, serde::Serialize)]
pub struct UsernamePasswordFormData {
    pub username: String,
    #[schema(value_type = String)]
    #[serde(serialize_with = "secret_string_to_string")]
    pub password: SecretString,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct PostNewsletterFormData {
    /// 제목
    pub title: String,
    /// 내용
    pub content: Content,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct Content {
    /// HTML 문서
    pub html: String,
    /// 일반 텍스트
    pub text: String,
}

/// 핸들러에 들어오는 가입 요청 데이터
#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct SubscribeFormData {
    pub name: String,
    pub email: String,
}
