use anyhow::Context;
use ar_entity::delegation_evidence::ResourceRule;
use ishare::delegation_evidence::verify_delegation_evidence;
use ishare::delegation_request::{DelegationRequest, DelegationTarget, ResourceTarget};
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::db::policy::{self as policy_store, InsertPolicySetWithPolicies, MatchingPolicySetRow};
use crate::error::{AppError, ExpectedError};
use crate::services::delegation::create_delegation_evidence;
use crate::TimeProvider;

use super::ishare_provider::SatelliteProvider;

pub async fn validate_policy_set_ishare_parties(
    now: chrono::DateTime<chrono::Utc>,
    args: &InsertPolicySetWithPolicies,
    ishare: std::sync::Arc<dyn SatelliteProvider>,
) -> Result<(), AppError> {
    ishare
        .validate_party(now, &args.target.access_subject)
        .await
        .map_err(|e| {
            AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: format!(
                    "Unable to verify access subject '{}' as valid iSHARE party",
                    &args.target.access_subject
                ),
                reason: format!("{:?}", e),
                metadata: None,
            })
        })?;

    ishare
        .validate_party(now, &args.policy_issuer)
        .await
        .map_err(|e| {
            AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: format!(
                    "Unable to verify policy issuer '{}' as valid iSHARE party",
                    &args.policy_issuer
                ),
                reason: format!("{:?}", e),
                metadata: None,
            })
        })?;

    for p in args.policies.iter() {
        for sp in p.target.environment.service_providers.iter() {
            ishare.validate_party(now, sp).await.map_err(|e| {
                AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: format!(
                        "Unable to verify service provider '{}' as valid iSHARE party",
                        &sp
                    ),
                    reason: format!("{:?}", e),
                    metadata: None,
                })
            })?;
        }
    }

    Ok(())
}

pub async fn insert_policy_set_with_policies(
    now: chrono::DateTime<chrono::Utc>,
    requester_company_id: &str,
    args: &InsertPolicySetWithPolicies,
    db: &DatabaseConnection,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    ishare: std::sync::Arc<dyn SatelliteProvider>,
) -> Result<Uuid, AppError> {
    validate_policy_set_ishare_parties(now, args, ishare).await?;

    let identifiers = args
        .policies
        .iter()
        .map(|p| p.target.resource.resource_type.clone())
        .collect();

    let access = verify_policy_set_access(
        requester_company_id,
        &PolicySetAction::Create,
        &args.policy_issuer,
        &args.target.access_subject,
        identifiers,
        client_eori,
        time_provider,
        &db,
    )
    .await
    .context("error verifying if access to create policy set")?;

    if !access {
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::FORBIDDEN,
            message: "not allowed to create policy set".to_owned(),
            reason: "not allowed to create policy set".to_owned(),
            metadata: None,
        }));
    }

    let policy_set_id = policy_store::insert_policy_set_with_policies(args, db)
        .await
        .context("Error inserting policy set with policies")?;

    Ok(policy_set_id)
}

pub async fn insert_policy_set_with_policies_admin(
    now: chrono::DateTime<chrono::Utc>,
    args: &InsertPolicySetWithPolicies,
    db: &DatabaseConnection,
    ishare: std::sync::Arc<dyn SatelliteProvider>,
) -> Result<Uuid, AppError> {
    validate_policy_set_ishare_parties(now, args, ishare).await?;

    let policy_set_id = policy_store::insert_policy_set_with_policies(args, db)
        .await
        .context("Error inserting policy set with policies")?;

    Ok(policy_set_id)
}

pub enum PolicySetAction {
    Read,
    Edit,
    Create,
    Delete,
}

impl PolicySetAction {
    fn to_string(&self) -> String {
        match self {
            Self::Read => "Read".to_owned(),
            Self::Edit => "Edit".to_owned(),
            Self::Create => "Create".to_owned(),
            Self::Delete => "Delete".to_owned(),
        }
    }
}

pub async fn verify_policy_set_access(
    requestor_company_id: &str,
    action: &PolicySetAction,
    policy_issuer: &str,
    access_subject: &str,
    resource_types: Vec<String>,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    db: &DatabaseConnection,
) -> anyhow::Result<bool> {
    tracing::info!(
        "requestor from '{}' requesting policy set access '{}' ",
        requestor_company_id,
        action.to_string()
    );

    if requestor_company_id == policy_issuer {
        tracing::info!("access granted because issuer matches requestor");
        return Ok(true);
    }

    if matches!(action, PolicySetAction::Read) && requestor_company_id == access_subject {
        tracing::info!("access granted for action read because access subject matches requestor");
        return Ok(true);
    }

    let delegation_request = DelegationRequest {
        policy_issuer: policy_issuer.to_string(),
        target: DelegationTarget {
            access_subject: requestor_company_id.to_string(),
        },
        policy_sets: vec![ishare::delegation_request::PolicySet {
            policies: vec![ishare::delegation_request::Policy {
                target: ResourceTarget {
                    actions: vec![action.to_string()],
                    resource: ishare::delegation_request::Resource {
                        identifiers: resource_types.clone(),
                        resource_type: "PDP.Policy".to_string(),
                        attributes: vec!["*".to_string()],
                    },
                    environment: Some(ishare::delegation_request::Environment {
                        service_providers: vec![client_eori.to_string()],
                    }),
                },
                rules: vec![ishare::delegation_request::ResourceRules {
                    effect: "Permit".to_string(),
                }],
            }],
        }],
    };

    tracing::info!(
        "checking if delegation evidence exists that '{}' can {} policies of types '{:?}' on behalf of '{}'. Service provider: '{}'",
        requestor_company_id,
        action.to_string(),
        &resource_types,
        policy_issuer,
        &client_eori,
    );

    let delegation_evidence_container =
        create_delegation_evidence(&delegation_request, time_provider, 30, db)
            .await
            .context("Error creating delegation evidence")?;

    let access = verify_delegation_evidence(
        &delegation_evidence_container.delegation_evidence,
        "PDP.Policy".to_owned(),
    );

    if access {
        tracing::info!("access granted because there is delegation evidence")
    } else {
        tracing::info!("access denied because there is not delegation evidence")
    }

    return Ok(access);
}

pub async fn delete_policy_set(
    requester_company_id: &str,
    id: &Uuid,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    db: &DatabaseConnection,
) -> Result<(), AppError> {
    let policy_set = match policy_store::get_policy_set_by_id(&id, &db)
        .await
        .context("Error getting policy set")?
    {
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::NOT_FOUND,
                message: "Can't find policy set".to_owned(),
                reason: "not found".to_owned(),
                metadata: None,
            }));
        }
        Some(ps) => ps,
    };

    let policies = policy_store::get_policies_by_policy_set(id, db)
        .await
        .context(format!(
            "Error getting policies from db for policy set: {}",
            id
        ))?;

    let identifiers = policies.iter().map(|p| p.resource_type.clone()).collect();

    let access = verify_policy_set_access(
        &requester_company_id,
        &PolicySetAction::Delete,
        &policy_set.policy_issuer,
        &policy_set.access_subject,
        identifiers,
        client_eori,
        time_provider,
        &db,
    )
    .await
    .context("error verifying if access to delete policy set")?;

    if !access {
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::FORBIDDEN,
            message: "not allowed to delete policy set".to_owned(),
            reason: "not allowed to delete policy set".to_owned(),
            metadata: None,
        }));
    }

    policy_store::delete_policy_set(&id, &db)
        .await
        .context(format!("Error deleting policy set: {}", id))?;

    Ok(())
}

pub async fn add_policy_to_policy_set(
    now: chrono::DateTime<chrono::Utc>,
    requester_company_id: &str,
    policy_set_id: &Uuid,
    policy: ar_entity::delegation_evidence::Policy,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    satellite_provider: std::sync::Arc<dyn SatelliteProvider>,
    db: &DatabaseConnection,
) -> Result<ar_entity::policy::Model, AppError> {
    match policy.rules.get(0) {
        Some(ResourceRule::Permit) => {}
        _ => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: "First rule must have effect 'Permit'".to_owned(),
                reason: "First rule must have effect 'Permit'".to_owned(),
                metadata: None,
            }))
        }
    }

    for sp in policy.target.environment.service_providers.iter() {
        satellite_provider
            .validate_party(now, sp)
            .await
            .map_err(|e| {
                AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: format!(
                        "Unable to verify service provider '{}' as valid iSHARE party",
                        &sp
                    ),
                    reason: format!("{:?}", e),
                    metadata: None,
                })
            })?;
    }

    let policy_set = match policy_store::get_policy_set_by_id(&policy_set_id, &db)
        .await
        .context("Error getting policy set")?
    {
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::NOT_FOUND,
                message: "Can't find policy set".to_owned(),
                reason: "not found".to_owned(),
                metadata: None,
            }));
        }
        Some(ps) => ps,
    };

    let policies = policy_store::get_policies_by_policy_set(policy_set_id, db)
        .await
        .context(format!(
            "Error getting policies from db for policy set: {}",
            policy_set_id
        ))?;

    let identifiers = policies.iter().map(|p| p.resource_type.clone()).collect();

    let access = verify_policy_set_access(
        &requester_company_id,
        &PolicySetAction::Edit,
        &policy_set.policy_issuer,
        &policy_set.access_subject,
        identifiers,
        client_eori,
        time_provider,
        &db,
    )
    .await
    .context("error verifying if access to edit policy set")?;

    if !access {
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::FORBIDDEN,
            message: "not allowed to edit policy set".to_owned(),
            reason: "not allowed to edit policy set".to_owned(),
            metadata: None,
        }));
    }

    let policy = policy_store::add_policy_to_policy_set(policy_set_id, policy, db)
        .await
        .context("Error adding policy to policy set")?;

    Ok(policy)
}

pub async fn replace_policy_in_policy_set(
    now: chrono::DateTime<chrono::Utc>,
    requester_company_id: &str,
    policy_set_id: Uuid,
    policy_id: Uuid,
    policy: ar_entity::delegation_evidence::Policy,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    satellite_provider: std::sync::Arc<dyn SatelliteProvider>,
    db: &DatabaseConnection,
) -> Result<ar_entity::policy::Model, AppError> {
    match policy.rules.get(0) {
        Some(ResourceRule::Permit) => {}
        _ => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: "First rule must have effect 'Permit'".to_owned(),
                reason: "First rule must have effect 'Permit'".to_owned(),
                metadata: None,
            }))
        }
    }

    for sp in policy.target.environment.service_providers.iter() {
        satellite_provider
            .validate_party(now, sp)
            .await
            .map_err(|e| {
                AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: format!(
                        "Unable to verify service provider '{}' as valid iSHARE party",
                        &sp
                    ),
                    reason: format!("{:?}", e),
                    metadata: None,
                })
            })?;
    }

    let policy_set = match policy_store::get_policy_set_by_id(&policy_set_id, &db)
        .await
        .context("Error getting policy set")?
    {
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::NOT_FOUND,
                message: "Can't find policy set".to_owned(),
                reason: "not found".to_owned(),
                metadata: None,
            }));
        }
        Some(ps) => ps,
    };

    let policies = policy_store::get_policies_by_policy_set(&policy_set_id, db)
        .await
        .context(format!(
            "Error getting policies from db for policy set: {}",
            policy_set_id
        ))?;

    let identifiers = policies.iter().map(|p| p.resource_type.clone()).collect();

    let access = verify_policy_set_access(
        &requester_company_id,
        &PolicySetAction::Edit,
        &policy_set.policy_issuer,
        &policy_set.access_subject,
        identifiers,
        client_eori,
        time_provider,
        &db,
    )
    .await
    .context("error verifying if access to edit policy set")?;

    if !access {
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::FORBIDDEN,
            message: "not allowed to edit policy set".to_owned(),
            reason: "not allowed to edit policy set".to_owned(),
            metadata: None,
        }));
    }

    let policy = policy_store::replace_policy(policy_set_id, policy_id, &policy, db)
        .await
        .context("Error adding policy to policy set")?;

    Ok(policy)
}

pub async fn get_policy_set_with_policies(
    requester_company_id: &str,
    policy_set_id: &Uuid,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    db: &DatabaseConnection,
) -> Result<Option<MatchingPolicySetRow>, AppError> {
    let policy_set = match policy_store::get_policy_set_by_id(&policy_set_id, &db)
        .await
        .context("Error getting policy set")?
    {
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::NOT_FOUND,
                message: "Can't find policy set".to_owned(),
                reason: "not found".to_owned(),
                metadata: None,
            }));
        }
        Some(ps) => ps,
    };

    let policies = policy_store::get_policies_by_policy_set(policy_set_id, db)
        .await
        .context(format!(
            "Error getting policies from db for policy set: {}",
            policy_set_id
        ))?;

    let identifiers = policies.iter().map(|p| p.resource_type.clone()).collect();

    let access = verify_policy_set_access(
        &requester_company_id,
        &PolicySetAction::Read,
        &policy_set.policy_issuer,
        &policy_set.access_subject,
        identifiers,
        client_eori,
        time_provider,
        &db,
    )
    .await
    .context("error verifying if access to read policy set")?;

    if !access {
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::FORBIDDEN,
            message: "not allowed to read policy set".to_owned(),
            reason: "not allowed to read policy set".to_owned(),
            metadata: None,
        }));
    }

    let ps = policy_store::get_policy_set_with_policies(policy_set_id, db).await?;

    Ok(ps)
}

pub async fn remove_policy_from_policy_set(
    requester_company_id: &str,
    policy_set_id: &Uuid,
    policy_id: &Uuid,
    client_eori: &str,
    time_provider: std::sync::Arc<dyn TimeProvider>,
    db: &DatabaseConnection,
) -> Result<(), AppError> {
    let policy_set = match policy_store::get_policy_set_by_id(&policy_set_id, &db)
        .await
        .context("Error getting policy set")?
    {
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::NOT_FOUND,
                message: "Can't find policy set".to_owned(),
                reason: "not found".to_owned(),
                metadata: None,
            }));
        }
        Some(ps) => ps,
    };

    let policies = policy_store::get_policies_by_policy_set(policy_set_id, db)
        .await
        .context(format!(
            "Error getting policies from db for policy set: {}",
            policy_set_id
        ))?;

    let policy = match policies.iter().find(|p| &p.id == policy_id) {
        Some(p) => p,
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::NOT_FOUND,
                message: "Can't find policy within policy set".to_owned(),
                reason: "Can't find policy within policy set".to_owned(),
                metadata: None,
            }))
        }
    };

    let identifiers = vec![policy.resource_type.to_owned()];

    let access = verify_policy_set_access(
        &requester_company_id,
        &PolicySetAction::Delete,
        &policy_set.policy_issuer,
        &policy_set.access_subject,
        identifiers,
        client_eori,
        time_provider,
        &db,
    )
    .await
    .context("error verifying if access to delete policy")?;

    if !access {
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::FORBIDDEN,
            message: "not allowed to delete policy".to_owned(),
            reason: "not allowed to delete policy".to_owned(),
            metadata: None,
        }));
    }

    policy_store::delete_policy(policy_id, db)
        .await
        .context("Error deleting policy")?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::*;
    use helpers::{init_test_db, FakeTimeProvider};
    use serde_json::json;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    #[sqlx::test]
    async fn test_verify_policy_set_access_pi_match(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let time_provider = std::sync::Arc::new(FakeTimeProvider::new());

        let access = verify_policy_set_access(
            "company",
            &PolicySetAction::Delete,
            "company",
            "as",
            vec!["*".to_owned()],
            "antother-company",
            time_provider,
            &db,
        )
        .await
        .unwrap();

        assert_eq!(access, true);

        Ok(())
    }

    #[sqlx::test]
    async fn test_verify_policy_set_access_no_pi_match(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let time_provider = std::sync::Arc::new(FakeTimeProvider::new());

        let access = verify_policy_set_access(
            "company",
            &PolicySetAction::Delete,
            "company-2",
            "as",
            vec!["*".to_owned()],
            "another-company",
            time_provider,
            &db,
        )
        .await
        .unwrap();

        assert_eq!(access, false);

        Ok(())
    }

    #[sqlx::test]
    async fn test_verify_policy_set_access_as_read(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let time_provider = std::sync::Arc::new(FakeTimeProvider::new());

        let access = verify_policy_set_access(
            "as-company",
            &PolicySetAction::Read,
            "company",
            "as-company",
            vec!["*".to_owned()],
            "antother-company",
            time_provider,
            &db,
        )
        .await
        .unwrap();

        assert_eq!(access, true);

        Ok(())
    }

    #[sqlx::test]
    async fn test_verify_policy_set_access_as_delete(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let time_provider = std::sync::Arc::new(FakeTimeProvider::new());

        let access = verify_policy_set_access(
            "as-company",
            &PolicySetAction::Delete,
            "company",
            "as-company",
            vec!["*".to_owned()],
            "antother-company",
            time_provider,
            &db,
        )
        .await
        .unwrap();

        assert_eq!(access, false);

        Ok(())
    }

    #[sqlx::test]
    async fn test_verify_policy_set_access_via_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let time_provider = std::sync::Arc::new(FakeTimeProvider::new());
        let policy_set: InsertPolicySetWithPolicies = serde_json::from_value(json!({
            "target": {
                "accessSubject": "NL.24244",
            },
            "policyIssuer": "NL.44444",
            "licences": ["ISHARE.0001"],
            "maxDelegationDepth": 2,
            "policies": [
                {
                    "target": {
                        "resource": {
                            "type": "PDP.Policy",
                            "identifiers": ["LovelyResource"],
                            "attributes": ["*"],
                        },
                        "actions": ["Delete"],
                        "environment": {
                            "serviceProviders": ["NL.CONSUME_TOO_MUCH"],
                        },
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        }
                    ]
                }
            ]
        }))
        .unwrap();
        policy_store::insert_policy_set_with_policies(&policy_set, &db)
            .await
            .unwrap();

        let access = verify_policy_set_access(
            "NL.24244",
            &PolicySetAction::Delete,
            "NL.44444",
            "as",
            vec!["LovelyResource".to_string()],
            "NL.CONSUME_TOO_MUCH",
            time_provider,
            &db,
        )
        .await
        .unwrap();

        assert_eq!(access, true);

        Ok(())
    }
}
