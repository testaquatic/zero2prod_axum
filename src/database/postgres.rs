pub mod subscription;

pub struct PostgresDatabase {
    pool: sqlx::PgPool,
}

impl PostgresDatabase {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
