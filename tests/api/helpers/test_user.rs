use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core},
};
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
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .expect("failed to hash password")
            .to_string();

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
