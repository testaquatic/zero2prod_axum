use sha3::Digest;
use uuid::Uuid;

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    pub async fn store(&self, pool: &sqlx::PgPool) {
        let password_hash = sha3::Sha3_256::digest(self.password.as_bytes());
        let password_hash = hex::encode(password_hash);

        sqlx::query!(
            r#"
            INSERT INTO users (user_id, username, password_hash, role)
            VALUES ($1, $2, $3, $4)
            "#,
            self.user_id,
            self.username,
            password_hash,
            "admin"
        )
        .execute(pool)
        .await
        .expect("failed to add test user");
    }
}
