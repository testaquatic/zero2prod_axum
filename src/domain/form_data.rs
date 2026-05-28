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
    /// HTML 문서
    #[schema(example = "<p>Newsletter body as HTML</p>")]
    pub html_content: String,
    /// 일반 텍스트
    #[schema(example = "Newsletter body as plain text")]
    pub text_content: String,
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
