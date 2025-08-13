use entities::m20250811_000001_add_status_to_subscriptions;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{
        ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, QueryFilter, StatementBuilder,
        TransactionTrait,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // https://github.com/SeaQL/sea-orm/blob/master/examples/seaography_example/migration/src/m20230102_000001_seed_bakery_data.rs 이 소스를 참고로 했다.
        let db = manager.get_connection();
        let status_confirmed =
            m20250811_000001_add_status_to_subscriptions::subscriptions::ActiveModel {
                status: sea_orm::ActiveValue::Set(Some("confirmed".into())),
                ..Default::default()
            };
        let transaction = db.begin().await?;

        m20250811_000001_add_status_to_subscriptions::subscriptions::Entity::update_many()
            .set(status_confirmed)
            .filter(
                m20250811_000001_add_status_to_subscriptions::subscriptions::Column::Status
                    .is_null(),
            )
            .exec(&transaction)
            .await?;
        transaction
            .execute(StatementBuilder::build(
                Table::alter()
                    .table(Subscriptions::Table)
                    .modify_column(ColumnDef::new(Subscriptions::Status).not_null()),
                &DatabaseBackend::Postgres,
            ))
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Status,
}
