use sea_orm_migration::prelude::*;

use crate::m20250127_143038_policy_set_template::PolicySetTemplate;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PolicySetTemplate::Table)
                    .add_column_if_not_exists(ColumnDef::new(Alias::new("description")).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PolicySetTemplate::Table)
                    .drop_column(Alias::new("description"))
                    .to_owned(),
            )
            .await
    }
}
