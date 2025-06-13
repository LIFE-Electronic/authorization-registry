use anyhow::Context;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

pub async fn get_all_policy_set_templates(
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<ar_entity::policy_set_template::Model>> {
    let policy_set_templates = ar_entity::policy_set_template::Entity::find()
        .all(db)
        .await
        .context("Error getting policy sets from database")?;

    return Ok(policy_set_templates);
}

pub async fn get_policy_set_template_by_id(
    id: &Uuid,
    db: &DatabaseConnection,
) -> anyhow::Result<Option<ar_entity::policy_set_template::Model>> {
    let ps_template = ar_entity::policy_set_template::Entity::find_by_id(*id)
        .one(db)
        .await?;

    Ok(ps_template)
}

#[derive(Deserialize, ToSchema)]
pub struct InsertPolicySetTemplate {
    pub policies: Vec<ar_entity::policy_set_template::Policy>,
    access_subject: Option<String>,
    policy_issuer: Option<String>,
    name: String,
    description: Option<String>
}

pub async fn insert_policy_set_template(
    new_ps_template: InsertPolicySetTemplate,
    db: &DatabaseConnection,
) -> anyhow::Result<Uuid> {
    let to_insert = ar_entity::policy_set_template::ActiveModel {
        id: sea_orm::ActiveValue::Set(uuid::Uuid::new_v4()),
        access_subject: sea_orm::ActiveValue::Set(new_ps_template.access_subject),
        policy_issuer: sea_orm::ActiveValue::Set(new_ps_template.policy_issuer),
        policies: sea_orm::ActiveValue::Set(new_ps_template.policies),
        name: sea_orm::ActiveValue::Set(new_ps_template.name),
        description: sea_orm::ActiveValue::Set(new_ps_template.description),
    };

    let inserted_id = ar_entity::policy_set_template::Entity::insert(to_insert)
        .exec(db)
        .await
        .context("Error inserting policy set template to db")?
        .last_insert_id;

    Ok(inserted_id)
}

pub async fn delete_policy_template(id: Uuid, db: &DatabaseConnection) -> anyhow::Result<()> {
    tracing::info!("Deleting policy set template with id: {}", &id);
    ar_entity::policy_set_template::Entity::delete_by_id(id)
        .exec(db)
        .await?;

    Ok(())
}
