use secrecy::SecretString;

use crate::domain::serializer::secret_string_to_string;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, serde::Serialize)]
pub struct LoginFormData {
    #[schema(example = "username")]
    pub username: String,
    #[schema(value_type = String, example = "password")]
    #[serde(serialize_with = "secret_string_to_string")]
    pub password: SecretString,
}

#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct PostNewsletterFormData {
    /// 제목
    #[schema(example = "제목")]
    pub title: String,
    /// 내용
    pub content: Content,
}

#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct Content {
    /// HTML 문서
    #[schema(example = "<h1>HTML 문서</h1>")]
    pub html: String,
    /// 일반 텍스트
    #[schema(example = "일반 텍스트")]
    pub text: String,
}

/// 핸들러에 들어오는 가입 요청 데이터
#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct SubscriptionFormData {
    #[schema(example = "Le Guin")]
    pub name: String,
    #[schema(example = "ursula_le_guin@gmail.com")]
    pub email: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordFormData {
    #[schema(value_type = String, example = "password")]
    pub current_password: SecretString,
    #[schema(value_type = String, example = "new_password")]
    pub new_password: SecretString,
    #[schema(value_type = String, example = "new_password")]
    pub new_password_check: SecretString,
}
