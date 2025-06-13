pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250127_143038_policy_set_template;
mod m20250613_125956_policy_set_template_description_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250127_143038_policy_set_template::Migration),
            Box::new(m20250613_125956_policy_set_template_description_column::Migration),
        ]
    }
}
