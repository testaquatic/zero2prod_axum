use entities::m20250811_000002_add_make_status_not_null_in_subscriptions;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(SubscriptionTokens::Table)
                .if_not_exists()
                .col(ColumnDef::new(SubscriptionTokens::SubscriptionToken).text().not_null())
                .col(ColumnDef::new(SubscriptionTokens::SubscriberID).uuid().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from_tbl(SubscriptionTokens::Table)
                        .from_col(SubscriptionTokens::SubscriberID)
                        .to_tbl(m20250811_000002_add_make_status_not_null_in_subscriptions::subscriptions::Entity)
                        .to_col(m20250811_000002_add_make_status_not_null_in_subscriptions::subscriptions::Column::Id)
                ).primary_key(Index::create().col(SubscriptionTokens::SubscriptionToken))
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(SubscriptionTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SubscriptionTokens {
    Table,
    SubscriptionToken,
    SubscriberID,
}
