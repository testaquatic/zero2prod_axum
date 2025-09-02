pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250811_000001_add_status_to_subscriptions;
mod m20250811_000002_add_make_status_not_null_in_subscriptions;
mod m20250811_000003_create_subscription_token_table;
mod m20250817_000001_create_users_table;
mod m20250818_000001_rename_password_column;
mod m20250818_000002_add_salt_to_users;
mod m20250818_000003_remove_salt_from_users;
mod m20250829_000001_seed_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250811_000001_add_status_to_subscriptions::Migration),
            Box::new(m20250811_000002_add_make_status_not_null_in_subscriptions::Migration),
            Box::new(m20250811_000003_create_subscription_token_table::Migration),
            Box::new(m20250817_000001_create_users_table::Migration),
            Box::new(m20250818_000001_rename_password_column::Migration),
            Box::new(m20250818_000002_add_salt_to_users::Migration),
            Box::new(m20250818_000003_remove_salt_from_users::Migration),
            Box::new(m20250829_000001_seed_user::Migration),
        ]
    }
}
