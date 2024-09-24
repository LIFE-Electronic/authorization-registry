use sea_orm;
use sea_orm::FromJsonQueryResult;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DelegationEvidence {
    not_before: i64,
    not_on_or_after: i64,
    policy_issuer: String,
    target: DelegationTarget,
    pub policy_sets: Vec<PolicySet>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DelegationTarget {
    access_subject: String,
}

#[derive(Serialize, Deserialize, Debug, FromQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct PolicySet {
    pub max_delegation_depth: i32,
    #[sea_orm(column_type = "Json")]
    pub target: PolicySetTarget,
    #[sea_orm(column_type = "Json")]
    pub policies: Vec<Policy>,
}

#[derive(Serialize, Deserialize, Debug, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetTarget {
    pub environment: PolicySetTargetEnvironment,
}

#[derive(Serialize, Deserialize, Debug, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct PolicySetTargetEnvironment {
    pub licenses: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, FromJsonQueryResult, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub target: ResourceTarget,
    pub rules: Vec<ResourceRule>,
}

#[derive(
    Serialize, Deserialize, Clone, FromJsonQueryResult, Debug, PartialEq, Eq, FromQueryResult,
)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTarget {
    pub resource: Resource,
    pub actions: Vec<String>,
    pub environment: Environment,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, FromJsonQueryResult)]
pub struct Resource {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub identifiers: Vec<String>,
    pub attributes: Vec<String>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, FromJsonQueryResult)]
pub struct Target {
    pub resource: Resource,
    pub actions: Vec<String>,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, FromJsonQueryResult)]
pub struct Deny {
    pub target: Target,
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug, FromJsonQueryResult)]
#[serde(tag = "effect")]
pub enum ResourceRule {
    Permit,
    Deny(Deny),
}

#[derive(Serialize, Deserialize, Clone, FromJsonQueryResult, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub service_providers: Vec<String>,
}
