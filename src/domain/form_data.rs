use secrecy::SecretString;
use uuid::Uuid;

use crate::domain::serializer::secret_string_to_string;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, serde::Serialize)]
pub struct LoginData {
    #[schema(example = "username")]
    pub username: String,
    #[schema(value_type = String, example = "password")]
    #[serde(serialize_with = "secret_string_to_string")]
    pub password: SecretString,
}

#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct PostNewsletterData {
    /// 제목
    #[schema(example = "제목")]
    pub title: String,
    /// HTML 문서
    #[schema(example = "<p>Newsletter body as HTML</p>")]
    pub html_content: String,
    /// 일반 텍스트
    #[schema(example = "Newsletter body as plain text")]
    pub text_content: String,
    /// 멱등성 키
    /// /admin/idempotency_key 에서 획득해야 하거나 무작위로 생성한 Uuid를 사용한다.
    /// 일회용이다.
    #[schema(value_type = String, example = "123e4567-e89b-12d3-a456-426655440000")]
    pub idempotency_key: Uuid,
}

/// 핸들러에 들어오는 가입 요청 데이터
#[derive(serde::Deserialize, utoipa::ToSchema, Debug)]
pub struct SubscriptionData {
    #[schema(example = "Le Guin")]
    pub name: String,
    #[schema(example = "ursula_le_guin@gmail.com")]
    pub email: String,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordData {
    #[schema(value_type = String, example = "password")]
    pub current_password: SecretString,
    #[schema(value_type = String, example = "new_password")]
    pub new_password: SecretString,
    #[schema(value_type = String, example = "new_password")]
    pub new_password_check: SecretString,
}
