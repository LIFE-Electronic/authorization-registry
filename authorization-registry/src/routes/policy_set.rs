use anyhow::Context;
use ar_entity::delegation_evidence::Policy;
use axum::extract::{Path, Query};
use axum::routing::delete;
use axum::{
    extract::State, middleware::from_fn_with_state, routing::post, Extension, Json, Router,
};
use axum_extra::extract::WithRejection;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::policy::{MatchingPolicySetRow, PolicySetsWithPagination};
use crate::error::{ErrorResponse, ExpectedError};
use crate::services::policy::{self as policy_service, InsertPolicySetWithPolicies};
use crate::{db::policy as policy_store, services::server_token::Role};
use crate::{error::AppError, AppState};
use crate::{middleware::extract_role_middleware, services::server_token::ServerToken};

pub fn get_policy_set_routes(server_token: Arc<ServerToken>) -> Router<AppState> {
    return Router::new()
        .route("/", post(insert_policy_set).get(get_all_policy_sets))
        .route(
            "/:id",
            delete(delete_policy_set)
                .get(get_policy_set)
                .put(put_policy_set),
        )
        .route("/:id/policy", post(add_policy_to_policy_set))
        .route(
            "/:id/policy/:policy_id",
            delete(delete_policy_from_policy_set).put(replace_policy_in_policy_set),
        )
        .layer(from_fn_with_state(server_token, extract_role_middleware));
}

#[derive(Deserialize)]
struct GetPolicySetsQuery {
    q: Option<String>,
    limit: Option<u32>,
    skip: Option<u32>,
}

/// Retrieve all policy sets belonging to the authenticated company
#[utoipa::path(
    get,
    path = "/policy-set",
    tag = "Policy Management",
    params(
        ("limit" = Option<u32>, Query, description = "Limit the number of results for pagination"),
        ("skip" = Option<u32>, Query, description = "Skip a number of results for pagination"),
        ("q" = Option<String>, Query, description = "Filter on any match in the policy set"),
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 200,
            description = "List of all policy sets and their associated policies",
            content_type = "application/json",
            body = Vec<MatchingPolicySetRow>
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json", 
            example = json!(ErrorResponse::new("Unauthorized")),
        )
    )
 )]
async fn get_all_policy_sets(
    Query(query): Query<GetPolicySetsQuery>,
    Extension(role): Extension<Role>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<PolicySetsWithPagination>, AppError> {
    let policy_sets = policy_store::get_policy_sets_with_policies(
        Some(role.get_company_id().to_string()),
        Some(role.get_company_id().to_string()),
        query.q,
        query.skip,
        query.limit,
        &db,
    )
    .await
    .context("Error getting policy sets")?;

    Ok(Json(policy_sets))
}

/// Remove a policy from a policy set
#[utoipa::path(
    delete,
    path = "/policy-set/{policy_set_id}/policy/{policy_id}",
    tag = "Policy Management",
    params(
        ("policy_set_id" = Uuid, Path, description = "Identifier of the policy set"),
        ("policy_id" = Uuid, Path, description = "Identifier of the policy to remove")
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 204,
            description = "Policy successfully removed from policy set"
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 403,
            description = "Forbidden",
            content_type = "application/json",
            example = json!(ErrorResponse::new("not allowed to delete policy"))
        ),
        (
            status = 404,
            description = "Policy set or policy not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Can't find policy within policy set"))
        )
    )
 )]
async fn delete_policy_from_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path((policy_set_id, policy_id)), _): WithRejection<Path<(Uuid, Uuid)>, AppError>,
    State(app_state): State<AppState>,
) -> Result<(), AppError> {
    policy_service::remove_policy_from_policy_set(
        app_state.time_provider.now(),
        &role.get_company_id(),
        &policy_set_id,
        &policy_id,
        &app_state.config.client_eori,
        app_state.time_provider,
        &db,
    )
    .await?;

    Ok(())
}

/// Retrieve a specific policy set by its ID
#[utoipa::path(
    get,
    path = "/policy-set/{id}",
    tag = "Policy Management",
    params(
        ("id" = Uuid, Path, description = "Unique identifier of the policy set to retrieve")
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 200,
            description = "Policy set successfully retrieved",
            content_type = "application/json",
            body = MatchingPolicySetRow
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 403,
            description = "Forbidden - insufficient permissions",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Not allowed to read policy set"))
        ),
        (
            status = 404,
            description = "Policy set not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Can't find policy set"))
        )
    )
)]
async fn get_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    Extension(role): Extension<Role>,
    State(app_state): State<AppState>,
) -> Result<Json<MatchingPolicySetRow>, AppError> {
    let ps = policy_service::get_policy_set_with_policies(
        &role.get_company_id(),
        &id,
        &app_state.config.client_eori,
        app_state.time_provider,
        &db,
    )
    .await?;

    match ps {
        Some(ps) => Ok(Json(ps)),
        None => Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::NOT_FOUND,
            message: "Can't find policy set".to_owned(),
            reason: "Can't find policy set".to_owned(),
            metadata: None,
        })),
    }
}

/// Replace an existing policy within a policy set
#[utoipa::path(
    put,
    path = "/policy-set/{policy_set_id}/policy/{policy_id}",
    tag = "Policy Management",
    params(
        ("policy_set_id" = Uuid, Path, description = "Identifier of the policy set"),
        ("policy_id" = Uuid, Path, description = "Identifier of the policy to replace")
    ),
    request_body(
        content = Policy,
        description = "New policy definition to replace the existing one",
        content_type = "application/json"
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 200,
            description = "Policy successfully replaced",
            content_type = "application/json",
            body = ar_entity::policy::Model
        ),
        (
            status = 400,
            description = "Invalid policy definition",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Invalid policy format"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 403,
            description = "Forbidden - insufficient permissions",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Not allowed to modify policy set"))
        ),
        (
            status = 404,
            description = "Policy set or policy not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Policy set or policy not found"))
        )
    )
)]
#[axum_macros::debug_handler]
async fn replace_policy_in_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path((policy_set_id, policy_id)), _): WithRejection<Path<(Uuid, Uuid)>, AppError>,
    State(app_state): State<AppState>,
    Json(body): Json<Policy>,
) -> Result<Json<ar_entity::policy::Model>, AppError> {
    let policy = policy_service::replace_policy_in_policy_set(
        app_state.time_provider.now(),
        &role.get_company_id(),
        policy_set_id,
        policy_id,
        body,
        &app_state.config.client_eori,
        app_state.time_provider,
        app_state.satellite_provider,
        &db,
    )
    .await?;

    Ok(Json(policy))
}

/// Add a new policy to an existing policy set
#[utoipa::path(
    post,
    path = "/policy-set/{id}/policy",
    tag = "Policy Management",
    params(
        ("id" = Uuid, Path, description = "Identifier of the policy set")
    ),
    request_body(
        content = Policy,
        description = "New policy definition to add to the policy set",
        content_type = "application/json"
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 200,
            description = "Policy successfully added to policy set",
            content_type = "application/json",
            body = ar_entity::policy::Model
        ),
        (
            status = 400,
            description = "Invalid policy definition",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Invalid policy format"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 403,
            description = "Forbidden - insufficient permissions",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Not allowed to modify policy set"))
        ),
        (
            status = 404,
            description = "Policy set not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Policy set not found"))
        )
    )
 )]
#[axum_macros::debug_handler]
async fn add_policy_to_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    State(app_state): State<AppState>,
    Json(body): Json<Policy>,
) -> Result<Json<ar_entity::policy::Model>, AppError> {
    let policy = policy_service::add_policy_to_policy_set(
        app_state.time_provider.now(),
        &role.get_company_id(),
        &id,
        body,
        &app_state.config.client_eori,
        app_state.time_provider,
        app_state.satellite_provider,
        &db,
    )
    .await?;

    Ok(Json(policy))
}

/// Delete a policy set and all its associated policies
#[utoipa::path(
    delete,
    path = "/policy-set/{id}",
    tag = "Policy Management",
    params(
        ("id" = Uuid, Path, description = "Identifier of the policy set to delete")
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 204,
            description = "Policy set successfully deleted"
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 403,
            description = "Forbidden - insufficient permissions", 
            content_type = "application/json",
            example = json!(ErrorResponse::new("Not allowed to delete policy set"))
        ),
        (
            status = 404,
            description = "Policy set not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Policy set not found"))
        )
    )
 )]
async fn delete_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    State(app_state): State<AppState>,
) -> Result<(), AppError> {
    policy_service::delete_policy_set(
        app_state.time_provider.now(),
        &role.get_company_id(),
        &id,
        &app_state.config.client_eori,
        app_state.time_provider,
        &db,
    )
    .await?;

    Ok(())
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct InsertPolicySetResponse {
    pub uuid: Uuid,
}

/// Create a new policy set with associated policies
#[utoipa::path(
    post,
    path = "/policy-set",
    tag = "Policy Management",
    request_body(
        content = InsertPolicySetWithPolicies,
        description = "Policy set details and its initial policies",
        content_type = "application/json"
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 201,
            description = "Policy set successfully created",
            content_type = "application/json",
            body = InsertPolicySetResponse
        ),
        (
            status = 400,
            description = "Invalid policy set or policy definitions",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Invalid policy set format"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 403,
            description = "Forbidden - insufficient permissions",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Not allowed to create policy sets"))
        )
    )
 )]
async fn insert_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<InsertPolicySetWithPolicies>, AppError>,
) -> Result<Json<InsertPolicySetResponse>, AppError> {
    let policy_set_id = policy_service::insert_policy_set_with_policies(
        app_state.time_provider.now(),
        &role.get_company_id(),
        &body,
        &db,
        &app_state.config.client_eori,
        app_state.time_provider.clone(),
        app_state.satellite_provider.clone(),
    )
    .await?;

    let response = InsertPolicySetResponse {
        uuid: policy_set_id,
    };

    Ok(Json(response))
}

/// Create or replace a policy set at a caller-supplied identifier
#[utoipa::path(
    put,
    path = "/policy-set/{id}",
    tag = "Policy Management",
    params(("id" = Uuid, Path, description = "Caller-supplied policy set identifier")),
    request_body(content = InsertPolicySetWithPolicies, content_type = "application/json"),
    security(("bearer" = [])),
    responses(
        (status = 201, description = "Policy set created", body = InsertPolicySetResponse),
        (status = 200, description = "Policy set replaced", body = InsertPolicySetResponse),
        (status = 400, description = "Invalid policy set or policy definitions", body = ErrorResponse),
        (status = 401, description = "Authentication failed", body = ErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ErrorResponse),
        (status = 409, description = "Policy set id is already in use by a different policy issuer", body = ErrorResponse)
    )
)]
async fn put_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    State(app_state): State<AppState>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    WithRejection(Json(body), _): WithRejection<Json<InsertPolicySetWithPolicies>, AppError>,
) -> Result<(StatusCode, Json<InsertPolicySetResponse>), AppError> {
    let existed = policy_service::put_policy_set_with_policies(
        app_state.time_provider.now(),
        id,
        &role.get_company_id(),
        &body,
        &db,
        &app_state.config.client_eori,
        app_state.time_provider.clone(),
        app_state.satellite_provider.clone(),
    )
    .await?;

    let status = if existed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, Json(InsertPolicySetResponse { uuid: id })))
}

#[cfg(test)]
mod test {
    use super::InsertPolicySetResponse;
    use crate::{fixtures::fixtures::insert_policy_set_fixture, services::server_token};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use reqwest::header::AUTHORIZATION;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use serde_json::json;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::super::super::test_helpers::helpers::*;

    fn policy_set_value_as_issuer(policy_issuer: &str, access_subject: &str) -> serde_json::Value {
        json!({
            "policies": [{
                "target": {
                    "resource": {
                        "type": "test-resource",
                        "identifiers": ["test-id"],
                        "attributes": ["*"]
                    },
                    "actions": ["Read"],
                    "environment": { "serviceProviders": ["nice-company"] }
                },
                "rules": [{ "effect": "Permit" }]
            }],
            "target": { "accessSubject": access_subject },
            "policyIssuer": policy_issuer,
            "licences": [],
            "maxDelegationDepth": 2
        })
    }

    fn policy_set_value(access_subject: &str) -> serde_json::Value {
        policy_set_value_as_issuer("nice-company", access_subject)
    }

    fn policy_set_body(access_subject: &str) -> Body {
        create_request_body(&policy_set_value(access_subject))
    }

    fn policy_set_body_as_issuer(policy_issuer: &str, access_subject: &str) -> Body {
        create_request_body(&policy_set_value_as_issuer(policy_issuer, access_subject))
    }

    fn policy_set_body_with_two_policies(access_subject: &str) -> Body {
        let mut body = policy_set_value(access_subject);
        let second_policy = body["policies"][0].clone();
        body["policies"].as_array_mut().unwrap().push(second_policy);
        create_request_body(&body)
    }

    #[sqlx::test]
    async fn put_policy_set_creates_and_replaces_at_the_supplied_id(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let policy_set_id = Uuid::new_v4();
        let authorization = server_token::server_token_test_helper::get_machine_token_header(Some(
            "nice-company".to_owned(),
        ));

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(AUTHORIZATION, &authorization)
                    .header("Content-Type", "application/json")
                    .body(policy_set_body_with_two_policies("first-subject"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let response_body = response.into_body().collect().await.unwrap().to_bytes();
        let response_body: InsertPolicySetResponse =
            serde_json::from_slice(&response_body).unwrap();
        assert_eq!(response_body.uuid, policy_set_id);
        let created = crate::db::policy::get_policy_set_by_id(&policy_set_id, &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(created.access_subject, "first-subject");
        assert_eq!(
            crate::db::policy::get_policies_by_policy_set(&policy_set_id, &db)
                .await
                .unwrap()
                .len(),
            2
        );

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(AUTHORIZATION, authorization)
                    .header("Content-Type", "application/json")
                    .body(policy_set_body("replacement-subject"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let replaced = crate::db::policy::get_policy_set_by_id(&policy_set_id, &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(replaced.access_subject, "replacement-subject");
        assert_eq!(replaced.created, created.created);

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "nice-company".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(policy_set_body("replacement-subject"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let policies = crate::db::policy::get_policies_by_policy_set(&policy_set_id, &db)
            .await
            .unwrap();
        assert_eq!(policies.len(), 1);

        let audit_events = ar_entity::audit_event::Entity::find()
            .filter(ar_entity::audit_event::Column::EntryId.eq(policy_set_id.to_string()))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(audit_events.len(), 3);
        assert_eq!(audit_events[0].event_type, "dmi:ar:policy_set:created");
        assert_eq!(audit_events[1].event_type, "dmi:ar:policy_set:edited");
        assert_eq!(
            audit_events[1].context.as_ref().unwrap()["edit_type"],
            "PolicySetReplaced"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn put_policy_set_rejects_a_caller_without_access(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let policy_set_id = Uuid::new_v4();

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "different-company".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(policy_set_body("subject"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(crate::db::policy::get_policy_set_by_id(&policy_set_id, &db)
            .await
            .unwrap()
            .is_none());
        Ok(())
    }

    #[sqlx::test]
    async fn put_policy_set_rejects_overwrite_of_foreign_policy_set(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        let foreign_policy_set_id =
            Uuid::parse_str("84b7fba4-05f3-4af8-9d84-dde384abe881").unwrap();

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{foreign_policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "EU.ATTACKER".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(policy_set_body_as_issuer("EU.ATTACKER", "EU.ATTACKER"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{foreign_policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "EU.ATTACKER".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(policy_set_body_as_issuer("NL.24244", "EU.ATTACKER"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let stored = crate::db::policy::get_policy_set_by_id(&foreign_policy_set_id, &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.policy_issuer, "NL.24244");
        assert_eq!(stored.access_subject, "NL.44444");
        Ok(())
    }

    #[sqlx::test]
    async fn put_policy_set_rejects_issuer_transfer(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let policy_set_id = Uuid::new_v4();

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "nice-company".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(policy_set_body("subject"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "nice-company-2".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(policy_set_body_as_issuer("nice-company-2", "subject"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let stored = crate::db::policy::get_policy_set_by_id(&policy_set_id, &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.policy_issuer, "nice-company");
        Ok(())
    }

    #[sqlx::test]
    async fn put_policy_set_allows_a_delegated_caller_to_replace(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        let foreign_policy_set_id =
            Uuid::parse_str("84b7fba4-05f3-4af8-9d84-dde384abe881").unwrap();
        let body = create_request_body(&json!({
            "policies": [{
                "target": {
                    "resource": { "type": "TestResource", "identifiers": ["id"], "attributes": ["*"] },
                    "actions": ["Read"],
                    "environment": { "serviceProviders": ["provider"] }
                },
                "rules": [{ "effect": "Permit" }]
            }],
            "target": { "accessSubject": "subject" },
            "policyIssuer": "NL.24244",
            "licences": [],
            "maxDelegationDepth": 2
        }));

        let response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{foreign_policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "NL.44444".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let stored = crate::db::policy::get_policy_set_by_id(&foreign_policy_set_id, &db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.access_subject, "subject");
        Ok(())
    }

    #[sqlx::test]
    async fn put_policy_set_allows_a_delegated_caller(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        let policy_set_id = Uuid::new_v4();
        let body = create_request_body(&json!({
            "policies": [{
                "target": {
                    "resource": { "type": "TestResource", "identifiers": ["id"], "attributes": ["*"] },
                    "actions": ["Read"],
                    "environment": { "serviceProviders": ["provider"] }
                },
                "rules": [{ "effect": "Permit" }]
            }],
            "target": { "accessSubject": "subject" },
            "policyIssuer": "NL.24244",
            "licences": [],
            "maxDelegationDepth": 2
        }));

        let response = get_test_app(db)
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{policy_set_id}"))
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "NL.44444".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(body)
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        Ok(())
    }

    #[sqlx::test]
    async fn put_policy_set_rejects_invalid_path_and_body(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let authorization = server_token::server_token_test_helper::get_machine_token_header(Some(
            "nice-company".to_owned(),
        ));

        let invalid_path = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/policy-set/not-a-uuid")
                    .method("PUT")
                    .header(AUTHORIZATION, &authorization)
                    .header("Content-Type", "application/json")
                    .body(policy_set_body("subject"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_path.status(), StatusCode::BAD_REQUEST);

        let invalid_body = get_test_app(db)
            .oneshot(
                Request::builder()
                    .uri(format!("/policy-set/{}", Uuid::new_v4()))
                    .method("PUT")
                    .header(AUTHORIZATION, authorization)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_body.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_set_by_id(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "NL.24244".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_policy_set(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db);

        let request_body = create_request_body(&json!(
            {
                "policies": [{
                    "target": {
                        "resource": {
                            "type": "test-iden2",
                            "identifiers": ["test", "test-2"],
                            "attributes": ["*"]
                        },
                        "actions": ["Read"],
                        "environment": {
                            "serviceProviders": ["asdf"]
                        }
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        }
                    ]
                }, {
                    "target": {
                        "resource": {
                            "type": "test-iden",
                            "identifiers": ["test4", "test-5"],
                            "attributes": ["*"]
                        },
                        "actions": ["*"],
                        "environment": {
                            "serviceProviders": ["fffd"]
                        }
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        },
                        {
                            "effect": "Deny",
                            "target": {
                                "resource": {
                                    "attributes": ["zinger"],
                                    "identifiers": ["*"],
                                    "type":"test-iden"
                                },
                                "actions": ["*"]
                            }
                        }
                    ]
                }],
                "target": {
                    "accessSubject": "sadfasdf"
                },
                "policyIssuer": "nice-company",
                "licences": [],
                "maxDelegationDepth": 2
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "nice-company".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_policy_set_different_policy_issuer_without_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db);

        let request_body = create_request_body(&json!(
            {
                "policies": [{
                    "target": {
                        "resource": {
                            "type": "test-iden2",
                            "identifiers": ["test", "test-2"],
                            "attributes": ["*"]
                        },
                        "actions": ["Read"],
                        "environment": {
                            "serviceProviders": ["asdf"]
                        }
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        }
                    ]
                }, {
                    "target": {
                        "resource": {
                            "type": "test-iden",
                            "identifiers": ["test4", "test-5"],
                            "attributes": ["*"]
                        },
                        "actions": ["*"],
                        "environment": {
                            "serviceProviders": ["fffd"]
                        }
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        },
                        {
                            "effect": "Deny",
                            "target": {
                                "resource": {
                                    "attributes": ["zinger"],
                                    "identifiers": ["*"],
                                    "type":"test-iden"
                                },
                                "actions": ["*"]
                            }
                        }
                    ]
                }],
                "target": {
                    "accessSubject": "sadfasdf"
                },
                "policyIssuer": "nice-company2",
                "licences": [],
                "maxDelegationDepth": 2
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "nice-company".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    #[sqlx::test]
    async fn test_insert_policy_set_different_policy_issuer_with_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!(
            {
                "policies": [{
                    "target": {
                        "resource": {
                            "type": "TestResource",
                            "identifiers": ["test", "test-2"],
                            "attributes": ["*"]
                        },
                        "actions": ["Read"],
                        "environment": {
                            "serviceProviders": ["asdf"]
                        }
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        }
                    ]
                }, {
                    "target": {
                        "resource": {
                            "type": "TestResource",
                            "identifiers": ["test4", "test-5"],
                            "attributes": ["*"]
                        },
                        "actions": ["*"],
                        "environment": {
                            "serviceProviders": ["fffd"]
                        }
                    },
                    "rules": [
                        {
                            "effect": "Permit"
                        },
                        {
                            "effect": "Deny",
                            "target": {
                                "resource": {
                                    "attributes": ["zinger"],
                                    "identifiers": ["*"],
                                    "type":"test-iden"
                                },
                                "actions": ["*"]
                            }
                        }
                    ]
                }],
                "target": {
                    "accessSubject": "sadfasdf"
                },
                "policyIssuer": "NL.24244",
                "licences": [],
                "maxDelegationDepth": 2
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_machine_token_header(Some(
                            "NL.44444".to_owned(),
                        )),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_policy_set(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881")
                    .method("DELETE")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.24244".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_policy_set_user_not_issuer(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881")
                    .method("DELETE")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("asdfasdf".to_owned()),
                            None,
                        ),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_policy_set_user_not_issuer_with_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881")
                    .method("DELETE")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_policy_to_policy_set(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "target": {
                "resource": {
                    "type": "test-iden2",
                    "identifiers": ["test", "test-2"],
                    "attributes": ["*"]
                },
                "actions": ["Read"],
                "environment": {
                    "serviceProviders": ["NL.EORI.LIFEELEC4DMI"]
                }
            },
            "rules": [
                {
                    "effect": "Permit"
                }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.24244".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_replace_policy_in_policy_set(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "target": {
                "resource": {
                    "type": "test-iden2",
                    "identifiers": ["test", "test-2"],
                    "attributes": ["*"]
                },
                "actions": ["Read"],
                "environment": {
                    "serviceProviders": ["NL.EORI.LIFEELEC4DMI"]
                }
            },
            "rules": [
                {
                    "effect": "Permit"
                }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy/564f3b46-7127-4c3c-a0b8-2859c01cc9c1")
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.24244".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_replace_policy_in_policy_set_no_first_effect_permit(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "target": {
                "resource": {
                    "type": "test-iden2",
                    "identifiers": ["test", "test-2"],
                    "attributes": ["*"]
                },
                "actions": ["Read"],
                "environment": {
                    "serviceProviders": ["NL.EORI.LIFEELEC4DMI"]
                }
            },
            "rules": [
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy/564f3b46-7127-4c3c-a0b8-2859c01cc9c1")
                    .method("PUT")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.24244".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_policy_to_policy_set_no_first_effect_permit(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "target": {
                "resource": {
                    "type": "test-iden2",
                    "identifiers": ["test", "test-2"],
                    "attributes": ["*"]
                },
                "actions": ["Read"],
                "environment": {
                    "serviceProviders": ["NL.EORI.LIFEELEC4DMI"]
                }
            },
            "rules": [
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.24244".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_policy_to_policy_set_no_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "target": {
                "resource": {
                    "type": "TestResource",
                    "identifiers": ["test", "test-2"],
                    "attributes": ["*"]
                },
                "actions": ["Read"],
                "environment": {
                    "serviceProviders": ["NL.EORI.LIFEELEC4DMI"]
                }
            },
            "rules": [
                {
                    "effect": "Permit"
                }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_policy_to_policy_set_via_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;

        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "target": {
                "resource": {
                    "type": "TestResource",
                    "identifiers": ["test", "test-2"],
                    "attributes": ["*"]
                },
                "actions": ["Read"],
                "environment": {
                    "serviceProviders": ["NL.EORI.LIFEELEC4DMI"]
                }
            },
            "rules": [
                {
                    "effect": "Permit"
                }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }

    #[sqlx::test]
    async fn test_remove_policy_from_policy_set(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy/564f3b46-7127-4c3c-a0b8-2859c01cc9c1")
                    .method("DELETE")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.24244".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }
}
