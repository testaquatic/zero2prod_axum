use sea_orm_migration::{
    async_trait,
    prelude::Table,
    sea_orm::{self, DeriveIden, DeriveMigrationName},
    DbErr, MigrationTrait, SchemaManager,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .rename_column(Users::Password, Users::PasswordHash)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Password,
    PasswordHash,
}
