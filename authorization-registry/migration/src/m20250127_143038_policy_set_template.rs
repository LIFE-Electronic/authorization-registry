use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum PolicySetTemplate {
    Table,
    Id,
    Name,
    AccessSubject,
    PolicyIssuer,
    Policies,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PolicySetTemplate::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PolicySetTemplate::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PolicySetTemplate::AccessSubject).text())
                    .col(ColumnDef::new(PolicySetTemplate::PolicyIssuer).text())
                    .col(ColumnDef::new(PolicySetTemplate::Name).text().not_null())
                    .col(
                        ColumnDef::new(PolicySetTemplate::Policies)
                            .json()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PolicySetTemplate::Table).to_owned())
            .await
    }
}
