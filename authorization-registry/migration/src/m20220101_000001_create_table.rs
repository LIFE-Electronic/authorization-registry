use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Company {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum IshareUser {
    Table,
    Id,
    Email,
    Fullname,
    Company,
    IdpUrl,
    IdpEori,
}

#[derive(DeriveIden)]
enum PolicySet {
    Table,
    Id,
    Licenses,
    PolicyIssuer,
    AccessSubject,
    MaxDelegationDepth,
}

#[derive(DeriveIden)]
enum Policy {
    Table,
    Id,
    Identifiers,
    Attributes,
    ServiceProviders,
    PolicySet,
    Actions,
    ResourceType,
    Rules,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Company::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Company::Id).text().primary_key().not_null())
                    .col(ColumnDef::new(Company::Name).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(IshareUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IshareUser::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(IshareUser::Fullname).text().not_null())
                    .col(ColumnDef::new(IshareUser::Email).text().not_null())
                    .col(ColumnDef::new(IshareUser::IdpEori).text().not_null())
                    .col(ColumnDef::new(IshareUser::IdpUrl).text().not_null())
                    .col(ColumnDef::new(IshareUser::Company).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ishare-user_company")
                            .from(IshareUser::Table, IshareUser::Company)
                            .to(Company::Table, Company::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PolicySet::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PolicySet::PolicyIssuer).text().not_null())
                    .col(ColumnDef::new(PolicySet::AccessSubject).text().not_null())
                    .col(
                        ColumnDef::new(PolicySet::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PolicySet::Licenses)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PolicySet::MaxDelegationDepth)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Policy::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Policy::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Policy::Identifiers)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Policy::ResourceType).text().not_null())
                    .col(
                        ColumnDef::new(Policy::Attributes)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Policy::Actions)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Policy::Rules).json().not_null())
                    .col(
                        ColumnDef::new(Policy::ServiceProviders)
                            .array(ColumnType::Text)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Policy::PolicySet).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-policy-policy_set")
                            .from(Policy::Table, Policy::PolicySet)
                            .to(PolicySet::Table, PolicySet::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IshareUser::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Company::Table).to_owned())
            .await
    }
}
