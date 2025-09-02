use sea_orm_migration::{
    async_trait,
    sea_orm::{self, DeriveMigrationName, EntityTrait},
    DbErr, MigrationTrait, SchemaManager,
};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]

impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let model = entities::m20250818_000003_remove_salt_from_users::users::ActiveModel {
            user_id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
            username: sea_orm::ActiveValue::Set("admin".to_string()),
            password_hash: sea_orm::ActiveValue::Set("$argon2id$v=19$m=19456,t=2,p=1$RC9zYW9TVGU$gT9hWRiaI2brDQr7nUhKNrLIvmFOlh2jhC5jGa8Ol+U".to_string()),
        };

        entities::m20250818_000003_remove_salt_from_users::users::Entity::insert(model)
            .exec(manager.get_connection())
            .await?;

        Ok(())
    }
}
