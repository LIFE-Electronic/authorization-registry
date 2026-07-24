use sea_orm_migration::prelude::*;

use crate::m20250619_124921_add_audit_log_table::AuditEvent;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AuditEvent::Table)
                    .add_column(
                        ColumnDef::new(AuditEvent::EntryId)
                            .text()
                            .default("".to_owned()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AuditEvent::Table)
                    .drop_column(AuditEvent::EntryId)
                    .to_owned(),
            )
            .await
    }
}
