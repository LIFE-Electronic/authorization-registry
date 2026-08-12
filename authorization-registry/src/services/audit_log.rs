use std::{collections::HashMap, fmt, sync::Arc};

use anyhow::Context;
use ar_entity::audit_event::{ActiveModel as AuditEventModel, Entity as AuditEventEntity};
use chrono::{DateTime, Utc};
use ishare::{
    delegation_evidence::verify_delegation_evidence,
    delegation_request::{
        DelegationRequest, DelegationTarget, Environment, Policy, PolicySet, Resource,
        ResourceRules, ResourceTarget,
    },
};
use reqwest::StatusCode;
use sea_orm::{
    ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    error::{AppError, ExpectedError},
    services::delegation::create_delegation_evidence,
    AppConfig, TimeProvider,
};

#[derive(Serialize, Deserialize)]
pub struct PolicySetCreatedEventMetadata {
    pub policy_set_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PolicyRemoved {
    pub policy_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PolicyAdded {
    pub policy_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct PolicyReplaced {
    pub old_policy_id: Uuid,
    pub new_policy_id: Uuid,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "edit_type")]
pub enum EditedType {
    PolicyRemoved(PolicyRemoved),
    PolicyAdded(PolicyAdded),
    PolicyReplaced(PolicyReplaced),
    PolicySetReplaced,
}

#[derive(Serialize, Deserialize)]
pub struct PolicySetEditedEventMetadata {
    pub policy_set_id: Uuid,
    #[serde(flatten)]
    pub edited_type: EditedType,
}

#[derive(Serialize, Deserialize)]
pub struct PolicySetDeletedEventMetadata {
    pub policy_set_id: Uuid,
}

pub enum EventType {
    DmiDelegationRequest(DelegationRequest),
    ArPolicySetCreated(PolicySetCreatedEventMetadata),
    ArPolicySetEdited(PolicySetEditedEventMetadata),
    ArPolicySetDeleted(PolicySetDeletedEventMetadata),
}

impl EventType {
    fn get_context(&self) -> anyhow::Result<Option<Value>> {
        match self {
            Self::DmiDelegationRequest(delegation_request) => Ok(Some(
                serde_json::to_value(delegation_request)
                    .context("Error parsing serde_json value")?,
            )),
            Self::ArPolicySetCreated(meta_data) => Ok(Some(
                serde_json::to_value(meta_data).context("Error parsing serde_json value")?,
            )),
            Self::ArPolicySetEdited(meta_data) => Ok(Some(
                serde_json::to_value(meta_data).context("Error parsing serde_json value")?,
            )),
            Self::ArPolicySetDeleted(meta_data) => Ok(Some(
                serde_json::to_value(meta_data).context("Error parsing serde_json value")?,
            )),
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EventType::DmiDelegationRequest(_) => "dmi:ar:delegation:request",
            EventType::ArPolicySetCreated(_) => "dmi:ar:policy_set:created",
            EventType::ArPolicySetEdited(_) => "dmi:ar:policy_set:edited",
            EventType::ArPolicySetDeleted(_) => "dmi:ar:policy_set:deleted",
        };
        write!(f, "{}", s)
    }
}

pub async fn log_event<T: ConnectionTrait>(
    now: DateTime<Utc>,
    entry_id: String,
    event_type: EventType,
    source: Option<String>,
    data: Option<Value>,
    db: &T,
) -> anyhow::Result<()> {
    let context = event_type.get_context()?;
    let event_type = event_type.to_string();
    let id = uuid::Uuid::new_v4();

    let log_entry = AuditEventModel {
        entry_id: ActiveValue::Set(entry_id.clone()),
        id: ActiveValue::Set(id.clone()),
        source: ActiveValue::Set(source),
        timestamp: ActiveValue::Set(now),
        event_type: ActiveValue::Set(event_type.clone()),
        context: ActiveValue::Set(context),
        data: ActiveValue::Set(data),
    };

    AuditEventEntity::insert(log_entry)
        .exec(db)
        .await
        .context("Error inserting audit log entry")?;

    tracing::info!("[{}] log entry saved with id -- {}", &event_type, &id);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuditEventWithIssAndSub {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub source: String,
    pub context: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    pub sub: String,
    pub iss: String,
    pub id: String,
}

fn add_entry_id_to_context(
    context: Option<serde_json::Value>,
    entry_id: String,
) -> HashMap<String, String> {
    let mut initial = match context {
        Some(context) => context.clone(),
        None => json!({}),
    };

    if let Some(obj) = initial.as_object_mut() {
        if entry_id.len() > 0 {
            obj.insert("entryId".to_string(), entry_id.to_string().into());
        }
    }

    let mut map = HashMap::new();
    if let Some(obj) = initial.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                map.insert(k.clone(), s.to_string());
            } else {
                map.insert(k.clone(), v.to_string());
            }
        }
    }
    return map;
}

fn add_iss_and_sub_and_id_to_context(
    client_eori: &str,
    controller_eori: &str,
    audit_event: ar_entity::audit_event::Model,
    source: &str,
) -> AuditEventWithIssAndSub {
    return AuditEventWithIssAndSub {
        timestamp: audit_event.timestamp,
        event_type: audit_event.event_type,
        source: source.to_owned(),
        context: add_entry_id_to_context(audit_event.context, audit_event.entry_id),
        data: audit_event.data,
        iss: client_eori.to_owned(),
        sub: controller_eori.to_owned(),
        id: audit_event.id.to_string(),
    };
}

pub async fn retrieve_events(
    controller_eori: &str,
    from: Option<chrono::DateTime<Utc>>,
    to: Option<chrono::DateTime<Utc>>,
    max_results: u64,
    event_types: Option<String>,
    time_provider: Arc<dyn TimeProvider>,
    app_config: &AppConfig,
    db: &DatabaseConnection,
) -> Result<Vec<AuditEventWithIssAndSub>, AppError> {
    tracing::info!(
        "checking if delegation evidence exists that '{}' can access the audit log",
        controller_eori
    );

    let delegation_evidence_container = create_delegation_evidence(
        &DelegationRequest {
            policy_issuer: app_config.client_eori.to_owned(),
            target: DelegationTarget {
                access_subject: controller_eori.to_string(),
            },
            policy_sets: vec![PolicySet {
                policies: vec![Policy {
                    target: ResourceTarget {
                        resource: Resource {
                            resource_type: "AuditLog".to_owned(),
                            identifiers: vec!["*".to_owned()],
                            attributes: vec!["*".to_owned()],
                        },
                        actions: vec!["Read".to_owned()],
                        environment: Some(Environment {
                            service_providers: vec![app_config.client_eori.to_string()],
                        }),
                    },
                    rules: vec![ResourceRules {
                        effect: "Permit".to_owned(),
                    }],
                }],
            }],
        },
        time_provider,
        30,
        db,
    )
    .await
    .context("Error creating delegation evidence")?;

    let access = verify_delegation_evidence(
        &delegation_evidence_container.delegation_evidence,
        "AuditLog".to_owned(),
    );

    if access {
        tracing::info!("access granted because there is delegation evidence")
    } else {
        tracing::info!("access denied because there is no delegation evidence");
        return Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::UNAUTHORIZED,
            message: "unauthorized".to_owned(),
            metadata: None,
            reason: "access denied: no delegation evidence exists".to_owned(),
        }));
    }

    let max_results = match max_results {
        mr if mr > 1000 => {
            tracing::info!(
                "max_results '{}' value higher than 1000, using 1000 instead",
                mr
            );
            1000
        }
        mr if mr < 1 => {
            tracing::info!("max_results '{}' value lower than 1, using 1 instead", mr);
            1
        }
        mr => mr,
    };

    let mut query = ar_entity::audit_event::Entity::find();

    if let Some(from) = from {
        query = query.filter(ar_entity::audit_event::Column::Timestamp.gte(from))
    }

    if let Some(to) = to {
        query = query.filter(ar_entity::audit_event::Column::Timestamp.lte(to))
    }

    if let Some(event_types) = event_types {
        let splitted_event_types: Vec<&str> = event_types.split(",").collect();

        let mut event_types_condition = Condition::any();

        for event_type in splitted_event_types {
            event_types_condition =
                event_types_condition.add(ar_entity::audit_event::Column::EventType.eq(event_type));
        }

        query = query.filter(event_types_condition);
    }

    let events = query
        .limit(max_results)
        .all(db)
        .await
        .context("Error retrieving audit log entries")?;

    let events_with_iss_and_sub: Vec<AuditEventWithIssAndSub> = events
        .into_iter()
        .map(|e| {
            add_iss_and_sub_and_id_to_context(
                &app_config.client_eori,
                controller_eori,
                e,
                &app_config.service_name,
            )
        })
        .collect();

    return Ok(events_with_iss_and_sub);
}

#[cfg(test)]

mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use crate::services::audit_log::add_entry_id_to_context;

    #[test]
    fn test_add_id_to_context() {
        let context = json!({
            "something": "whatever"
        });

        let id = Uuid::new_v4().to_string();
        let context = add_entry_id_to_context(Some(context), id.clone());

        assert_eq!(context.get("entryId").unwrap(), &id);
        assert_eq!(context.get("something").unwrap(), "whatever");
    }
}
