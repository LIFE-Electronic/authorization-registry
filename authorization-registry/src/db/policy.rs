use anyhow::{bail, Context};
use ar_entity::delegation_evidence::{Policy, ResourceRule};
use chrono::Utc;
use sea_orm::{self, ConnectionTrait, QueryFilter, TransactionTrait};
use sea_orm::{
    entity::*, DatabaseConnection, EntityTrait, FromJsonQueryResult, FromQueryResult, JsonValue,
    Statement,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub async fn get_policy(
    policy_set_id: Uuid,
    policy_id: Uuid,
    db: &DatabaseConnection,
) -> anyhow::Result<Option<ar_entity::policy::Model>> {
    let policy = ar_entity::policy::Entity::find()
        .filter(ar_entity::policy::Column::Id.eq(policy_id))
        .filter(ar_entity::policy::Column::PolicySet.eq(policy_set_id))
        .one(db)
        .await
        .context("Error fetching policy from db")?;

    Ok(policy)
}

pub async fn _get_all_policies(
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<ar_entity::policy::Model>> {
    let policies = ar_entity::policy::Entity::find()
        .all(db)
        .await
        .context("Error retrieving policies from db")?;

    return Ok(policies);
}

pub async fn _get_all_policy_sets(
    access_subject: Option<String>,
    policy_issuer: Option<String>,
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<ar_entity::policy_set::Model>> {
    let mut query = ar_entity::policy_set::Entity::find();

    if let Some(access_subject) = access_subject {
        query = query.filter(ar_entity::policy_set::Column::AccessSubject.eq(access_subject))
    }

    if let Some(policy_issuer) = policy_issuer {
        query = query.filter(ar_entity::policy_set::Column::PolicyIssuer.eq(policy_issuer))
    }

    let policy_sets = query
        .all(db)
        .await
        .context("error retrieving policy sets from db")?;

    return Ok(policy_sets);
}

pub async fn get_policies_by_policy_set(
    policy_set_id: &Uuid,
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<ar_entity::policy::Model>> {
    let policies = ar_entity::policy::Entity::find()
        .filter(ar_entity::policy::Column::PolicySet.eq(*policy_set_id))
        .all(db)
        .await
        .context("Error getting policies from db")?;

    Ok(policies)
}

#[derive(FromJsonQueryResult, Serialize, Deserialize, Debug, ToSchema)]
pub struct DelegationEvidencePolicy {
    pub id: Uuid,
    pub identifiers: Vec<String>,
    pub resource_type: String,
    pub attributes: Vec<String>,
    pub actions: Vec<String>,
    pub service_providers: Vec<String>,
    pub rules: Vec<ResourceRule>,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult, ToSchema)]
pub struct MatchingPolicySetRow {
    pub policy_set_id: Uuid,
    pub access_subject: String,
    pub policy_issuer: String,
    pub policies: Vec<DelegationEvidencePolicy>,
    pub licenses: Vec<String>,
    pub max_delegation_depth: i32,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
struct Pagination {
    total_count: i64,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct PolicySetsWithPagination {
    pub data: Vec<MatchingPolicySetRow>,
    pagination: Pagination,
}

#[derive(Debug, FromQueryResult)]
struct Count {
    count: i64,
}

pub async fn get_total_number_of_policy_sets(
    access_subject: Option<String>,
    policy_issuer: Option<String>,
    db: &DatabaseConnection,
) -> anyhow::Result<i64> {
    let mut conditions = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(access_subject) = access_subject {
        conditions.push(format!("access_subject like ${}", values.len() + 1));
        values.push(format!("%{}%", &access_subject).into());
    }

    if let Some(policy_issuer) = policy_issuer {
        conditions.push(format!("policy_issuer like ${}", values.len() + 1));
        values.push(format!("%{}%", &policy_issuer).into());
    }

    let condition = if conditions.len() > 0 {
        let joined_conditions: String = conditions.join(" and ");
        format!("where ({joined_conditions})")
    } else {
        "".to_owned()
    };

    let sql = format!(
        r#"
        SELECT
            COUNT(*) as count
        FROM
            policy_set ps
        {}
        "#,
        condition
    );

    tracing::info!("condition: {} {:?}", condition, values);

    let stmt = Statement::from_sql_and_values(sea_orm::DatabaseBackend::Postgres, sql, values);

    let result = Count::find_by_statement(stmt)
        .one(db)
        .await
        .context("Error fetching number of policy sets")?
        .expect("At least one row with the count");

    Ok(result.count)
}

pub async fn get_policy_sets_with_policies(
    access_subject: Option<String>,
    policy_issuer: Option<String>,
    skip: Option<u32>,
    limit: Option<u32>,
    db: &DatabaseConnection,
) -> anyhow::Result<PolicySetsWithPagination> {
    let mut conditions = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(access_subject) = &access_subject {
        conditions.push(format!("access_subject like ${}", values.len() + 1));
        values.push(format!("%{}%", access_subject).into());
    }

    if let Some(policy_issuer) = &policy_issuer {
        conditions.push(format!("policy_issuer like ${}", values.len() + 1));
        values.push(format!("%{}%", policy_issuer).into());
    }

    let condition = if conditions.len() > 0 {
        let joined_conditions: String = conditions.join(" and ");
        format!("where ({joined_conditions})")
    } else {
        "".to_owned()
    };

    let mut paginations = Vec::new();

    if let Some(limit) = limit {
        paginations.push(format!(" LIMIT ${}", values.len() + 1));
        values.push(limit.into());
    }

    if let Some(skip) = skip {
        paginations.push(format!(" OFFSET ${}", values.len() + 1));
        values.push(skip.into());
    }

    let pagination = paginations.join(" ");

    let sql = format!(
        r#"
            select
            ps.id as policy_set_id,
            ps.access_subject as access_subject,
            ps.policy_issuer as policy_issuer,
            ps.licenses as licenses,
            ps.max_delegation_depth as max_delegation_depth,
            coalesce(
                array_agg(
                    json_build_object(
                        'id',
                        p.id,
                        'identifiers',
                        p.identifiers,
                        'attributes',
                        p.attributes,
                        'actions',
                        p.actions,
                        'service_providers',
                        p.service_providers,
                        'resource_type',
                        p.resource_type,
                        'rules',
                        p.rules
                    )
                ) filter (where p.id is not null),
                '{{}}'
            ) as policies
        from
            policy_set ps
        left join
            policy p
                on p.policy_set = ps.id
        {}
        group by
            ps.id
        order by
            ps.created
            desc
        {}
    "#,
        condition, pagination,
    );

    let stmt =
        Statement::from_sql_and_values(sea_orm::DatabaseBackend::Postgres, sql, values.clone());

    let raw_result = JsonValue::find_by_statement(stmt)
        .all(db)
        .await
        .context("Error fetching policy sets from database")?;

    let policy_sets_parse_result: Result<Vec<MatchingPolicySetRow>, serde_json::Error> = raw_result
        .iter()
        .map(|r| serde_json::from_value::<MatchingPolicySetRow>(r.to_owned()))
        .collect();

    let policy_sets = policy_sets_parse_result
        .context("Error parsing policy sets 'QueryResult' into 'MatchingPolicySetRow'")?;

    let total_count = get_total_number_of_policy_sets(access_subject, policy_issuer, db)
        .await
        .context("Error getting total number of policy sets")?;

    Ok(PolicySetsWithPagination {
        data: policy_sets,
        pagination: Pagination { total_count },
    })
}

pub async fn get_own_policy_sets_with_policies(
    eori: &str,
    db: &DatabaseConnection,
) -> anyhow::Result<Vec<MatchingPolicySetRow>> {
    let mut conditions = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    conditions.push(format!("access_subject like ${}", values.len() + 1));
    values.push(format!("%{}%", &eori).into());

    conditions.push(format!("policy_issuer like ${}", values.len() + 1));
    values.push(format!("%{}%", &eori).into());

    let condition = if conditions.len() > 0 {
        let joined_conditions: String = conditions.join(" or ");
        format!("where ({joined_conditions})")
    } else {
        "".to_owned()
    };

    let sql = format!(
        r#"
            select
            ps.id as policy_set_id,
            ps.access_subject as access_subject,
            ps.policy_issuer as policy_issuer,
            ps.licenses as licenses,
            ps.max_delegation_depth as max_delegation_depth,
            coalesce(
                array_agg(
                    json_build_object(
                        'id',
                        p.id,
                        'identifiers',
                        p.identifiers,
                        'attributes',
                        p.attributes,
                        'actions',
                        p.actions,
                        'service_providers',
                        p.service_providers,
                        'resource_type',
                        p.resource_type,
                        'rules',
                        p.rules
                    )
                ) filter (where p.id is not null),
                '{{}}'
            ) as policies
        from
            policy_set ps
        left join
            policy p
                on p.policy_set = ps.id
        {}
        group by
            ps.id
    "#,
        condition
    );

    let stmt = Statement::from_sql_and_values(sea_orm::DatabaseBackend::Postgres, sql, values);

    let raw_result = JsonValue::find_by_statement(stmt)
        .all(db)
        .await
        .context("Error fetching policy sets from database")?;

    let policy_sets_parse_result: Result<Vec<MatchingPolicySetRow>, serde_json::Error> = raw_result
        .iter()
        .map(|r| serde_json::from_value::<MatchingPolicySetRow>(r.to_owned()))
        .collect();

    let policy_sets = policy_sets_parse_result
        .context("Error parsing policy sets 'QueryResult' into 'MatchingPolicySetRow'")?;

    Ok(policy_sets)
}

pub async fn get_policy_set_with_policies(
    policy_set_id: &Uuid,
    db: &DatabaseConnection,
) -> anyhow::Result<Option<MatchingPolicySetRow>> {
    let sql = r#"
            select
            ps.id as policy_set_id,
            ps.access_subject as access_subject,
            ps.policy_issuer as policy_issuer,
            ps.licenses as licenses,
            ps.max_delegation_depth as max_delegation_depth,
            coalesce(
                array_agg(
                    json_build_object(
                        'id',
                        p.id,
                        'identifiers',
                        p.identifiers,
                        'attributes',
                        p.attributes,
                        'actions',
                        p.actions,
                        'service_providers',
                        p.service_providers,
                        'resource_type',
                        p.resource_type,
                        'rules',
                        p.rules
                    )
                ) filter (where p.id is not null),
                '{}'
            ) as policies
        from
            policy_set ps
        left join
            policy p
                on p.policy_set = ps.id
        where (
            ps.id = $1
        )
        group by
            ps.id
    "#;

    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        sql,
        vec![policy_set_id.to_owned().into()],
    );

    let raw_result = JsonValue::find_by_statement(stmt)
        .one(db)
        .await
        .context("Error fetching policy set from database")?;

    let policy_set_option = match raw_result {
        Some(r) => Some(
            serde_json::from_value::<MatchingPolicySetRow>(r.to_owned())
                .context("Error parsing policy set 'QueryResult' into 'MatchingPolicySetRow'")?,
        ),
        None => None,
    };

    Ok(policy_set_option)
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessSubjectTarget {
    pub access_subject: String,
}

pub async fn insert_policy_set<C: ConnectionTrait>(
    now: chrono::DateTime<Utc>,
    target: &AccessSubjectTarget,
    policy_issuer: &str,
    licences: &Vec<String>,
    max_delegation_depth: &i32,
    db: &C,
) -> anyhow::Result<Uuid> {
    let policy_set_id = Uuid::new_v4();

    let active_policy_set = ar_entity::policy_set::ActiveModel {
        id: sea_orm::ActiveValue::Set(policy_set_id),
        licenses: sea_orm::ActiveValue::Set(licences.clone()),
        access_subject: sea_orm::ActiveValue::set(target.access_subject.clone()),
        policy_issuer: sea_orm::ActiveValue::set(policy_issuer.to_owned()),
        max_delegation_depth: sea_orm::ActiveValue::set(max_delegation_depth.to_owned()),
        created: sea_orm::ActiveValue::set(now),
    };

    let policy_set_id = ar_entity::policy_set::Entity::insert(active_policy_set)
        .exec(&*db)
        .await
        .context("Error inserting policy set into db")?
        .last_insert_id;

    Ok(policy_set_id)
}

pub async fn insert_policy<C: ConnectionTrait>(
    policy_set_id: Uuid,
    policy: &ar_entity::delegation_evidence::Policy,
    db: &C,
) -> anyhow::Result<Uuid> {
    let policy_id = Uuid::new_v4();

    let active_policy = ar_entity::policy::ActiveModel {
        id: sea_orm::ActiveValue::set(policy_id),
        attributes: sea_orm::ActiveValue::set(policy.target.resource.attributes.clone()),
        identifiers: sea_orm::ActiveValue::set(policy.target.resource.identifiers.clone()),
        service_providers: sea_orm::ActiveValue::set(
            policy.target.environment.service_providers.clone(),
        ),
        policy_set: sea_orm::ActiveValue::set(policy_set_id),
        actions: sea_orm::ActiveValue::set(policy.target.actions.clone()),
        resource_type: sea_orm::ActiveValue::set(policy.target.resource.resource_type.clone()),
        rules: sea_orm::ActiveValue::set(policy.rules.clone()),
    };

    let policy_id = ar_entity::policy::Entity::insert(active_policy)
        .exec(&*db)
        .await
        .context("Error inserting policy into db")?
        .last_insert_id;

    Ok(policy_id)
}

pub async fn replace_policy<C: ConnectionTrait>(
    policy_set_id: Uuid,
    policy_id: Uuid,
    new_policy: &ar_entity::delegation_evidence::Policy,
    db: &C,
) -> anyhow::Result<ar_entity::policy::Model> {
    let policy = ar_entity::policy::Entity::find_by_id(policy_id)
        .filter(ar_entity::policy::Column::PolicySet.eq(policy_set_id))
        .one(db)
        .await
        .context("Error retrieving policy from db")?;

    let mut active_policy = match policy {
        None => bail!("policy with id '{}' not found", policy_id),
        Some(policy) => policy.into_active_model(),
    };

    active_policy.attributes = ActiveValue::set(new_policy.target.resource.attributes.clone());
    active_policy.identifiers = ActiveValue::set(new_policy.target.resource.identifiers.clone());
    active_policy.service_providers =
        ActiveValue::set(new_policy.target.environment.service_providers.clone());
    active_policy.actions = ActiveValue::set(new_policy.target.actions.clone());
    active_policy.resource_type =
        ActiveValue::set(new_policy.target.resource.resource_type.clone());
    active_policy.rules = ActiveValue::set(new_policy.rules.clone());

    let active_policy = active_policy
        .save(db)
        .await
        .context("Error saving update policy to db")?;

    let policy = active_policy.try_into_model()?;

    Ok(policy)
}

pub async fn delete_policy_set(
    policy_set_id: &Uuid,
    db: &DatabaseConnection,
) -> anyhow::Result<()> {
    let transaction = db.begin().await.context("Error opening db transaction")?;

    ar_entity::policy::Entity::delete_many()
        .filter(ar_entity::policy::Column::PolicySet.eq(*policy_set_id))
        .exec(&transaction)
        .await
        .context(format!(
            "Error deleting policies for policy set: {}",
            policy_set_id
        ))?;

    ar_entity::policy_set::Entity::delete_by_id(*policy_set_id)
        .exec(&transaction)
        .await
        .context(format!("Error deleting policy set: {}", policy_set_id))?;

    transaction
        .commit()
        .await
        .context("Error commiting transaction to database")?;

    Ok(())
}

pub async fn get_policy_set_by_id(
    id: &Uuid,
    db: &DatabaseConnection,
) -> anyhow::Result<Option<ar_entity::policy_set::Model>> {
    ar_entity::policy_set::Entity::find_by_id(*id)
        .one(db)
        .await
        .context(format!("Error retrieving from db policy set: {}", id))
}

pub async fn add_policy_to_policy_set(
    policy_set_id: &Uuid,
    policy_args: Policy,
    db: &DatabaseConnection,
) -> anyhow::Result<ar_entity::policy::Model> {
    let id = uuid::Uuid::new_v4();

    let active_policy = ar_entity::policy::ActiveModel {
        id: sea_orm::ActiveValue::set(id),
        identifiers: sea_orm::ActiveValue::set(policy_args.target.resource.identifiers),
        attributes: sea_orm::ActiveValue::set(policy_args.target.resource.attributes),
        actions: sea_orm::ActiveValue::set(policy_args.target.actions),
        resource_type: sea_orm::ActiveValue::set(policy_args.target.resource.resource_type),
        rules: sea_orm::ActiveValue::set(policy_args.rules),
        service_providers: sea_orm::ActiveValue::set(
            policy_args.target.environment.service_providers,
        ),
        policy_set: sea_orm::ActiveValue::set(*policy_set_id),
    };

    let result = active_policy.insert(db).await.context(format!(
        "Error inserting new policy into db for policy set '{}'",
        policy_set_id
    ))?;

    Ok(result)
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InsertPolicySetWithPolicies {
    pub target: AccessSubjectTarget,
    pub policy_issuer: String,
    licences: Vec<String>,
    pub policies: Vec<ar_entity::delegation_evidence::Policy>,
    max_delegation_depth: i32,
}

pub async fn insert_policy_set_with_policies(
    now: chrono::DateTime<Utc>,
    args: &InsertPolicySetWithPolicies,
    db: &DatabaseConnection,
) -> anyhow::Result<Uuid> {
    let transaction = db.begin().await.context("Error opening db transaction")?;

    let policy_set_id = insert_policy_set(
        now,
        &args.target,
        &args.policy_issuer,
        &args.licences,
        &args.max_delegation_depth,
        &transaction,
    )
    .await
    .context("Error inserting policy set into db")?;

    for policy in args.policies.iter() {
        insert_policy(policy_set_id, &policy, &transaction)
            .await
            .context("Error inserting policy into db")?;
    }

    transaction
        .commit()
        .await
        .context("Error commiting transaction to db")?;

    Ok(policy_set_id)
}

pub async fn delete_policy(id: &Uuid, db: &DatabaseConnection) -> anyhow::Result<()> {
    ar_entity::policy::Entity::delete_by_id(*id)
        .exec(db)
        .await
        .context("Error deleting policy")?;

    Ok(())
}
