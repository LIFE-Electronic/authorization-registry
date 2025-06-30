use anyhow::Context;
use ar_entity::delegation_evidence::Policy;
use axum::{
    extract::{Path, Query, State},
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post},
    Extension, Json, Router,
};
use axum_extra::extract::WithRejection;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    db::policy::{self as policy_store, MatchingPolicySetRow, PolicySetsWithPagination},
    error::ExpectedError,
};
use crate::{db::policy_set_template::InsertPolicySetTemplate, services::policy as policy_service};
use crate::{error::AppError, error::ErrorResponse, AppState};
use crate::{
    middleware::{auth_role_middleware, extract_human_middleware, extract_role_middleware},
    services::server_token::ServerToken,
};

pub fn get_admin_routes(server_token: Arc<ServerToken>) -> Router<AppState> {
    return Router::new()
        .route(
            "/policy-set-template/:id",
            delete(delete_policy_set_template),
        )
        .route("/policy-set-template", post(insert_policy_set_template))
        .route(
            "/policy-set",
            post(insert_policy_set).get(get_all_policy_sets),
        )
        .route(
            "/policy-set/:id",
            get(get_policy_set).delete(delete_policy_set),
        )
        .route("/policy-set/:id/policy", post(add_policy_to_policy_set))
        .route(
            "/policy-set/:id/policy/:policy_id",
            delete(delete_policy_from_policy_set)
                .put(replace_policy_in_policy_set)
                .get(get_policy),
        )
        .layer(from_fn_with_state(
            vec!["dexspace_admin".to_owned()],
            auth_role_middleware,
        ))
        .layer(from_fn(extract_human_middleware))
        .layer(from_fn_with_state(server_token, extract_role_middleware));
}

#[utoipa::path(
    delete,
    path = "/admin/policy-set-template/{policy_set_template_id}",
    tag = "Policy Set Template - Admin",
    params(
        ("id" = Uuid, Path, description = "Identifier of the policy set template")
    ),
    security(
        ("h2m_bearer_admin" = [])
    ),
    responses(
        (
            status = 200,
            description = "Policy set template successfully deleted",
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        )
    )
 )]
async fn delete_policy_set_template(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<(), AppError> {
    crate::db::policy_set_template::delete_policy_template(id, &db).await?;

    Ok(())
}

#[derive(Deserialize, Serialize, ToSchema)]
struct InsertPolicySetTemplateResponse {
    uuid: Uuid,
}

#[utoipa::path(
    post,
    path = "/admin/policy-set-template",
    tag = "Policy Set Template - Admin",
    request_body(
        content = InsertPolicySetTemplate,
        description = "Insert new policy set template. Policy set templates can be used during the creation of new policy sets.",
        content_type = "application/json"
    ),
    security(
        ("h2m_bearer_admin" = [])
    ),
    responses(
        (
            status = 201,
            description = "Policy set template successfully created",
            content_type = "application/json",
            body = InsertPolicySetTemplateResponse
        ),
        (
            status = 400,
            description = "Invalid policy set template definition",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Invalid policy set template format"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        )
    )
 )]
async fn insert_policy_set_template(
    Extension(db): Extension<DatabaseConnection>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<InsertPolicySetTemplate>, AppError>,
) -> Result<Json<InsertPolicySetTemplateResponse>, AppError> {
    for p in body.policies.iter() {
        for sp in p.service_providers.iter() {
            app_state
                .satellite_provider
                .validate_party(app_state.time_provider.now(), sp)
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
    }

    let inserted_id = crate::db::policy_set_template::insert_policy_set_template(body, &db).await?;
    let response = InsertPolicySetTemplateResponse { uuid: inserted_id };

    Ok(Json(response))
}

/// Retrieve a specific policy within a policy set
#[utoipa::path(
    get,
    path = "/admin/policy-set/{policy_set_id}/policy/{policy_id}",
    tag = "Policy Management (Admin)",
    params(
        ("policy_set_id" = Uuid, Path, description = "Identifier of the policy set"),
        ("policy_id" = Uuid, Path, description = "Identifier of the policy to retrieve")
    ),
    security(
        ("h2m_bearer_admin" = [])
    ),
    responses(
        (
            status = 200,
            description = "Policy successfully retrieved",
            content_type = "application/json",
            body = ar_entity::policy::Model
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 404,
            description = "Policy or policy set not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Can't find policy"))
        )
    )
 )]
#[axum_macros::debug_handler]
async fn get_policy(
    Extension(db): Extension<DatabaseConnection>,
    WithRejection(Path((policy_set_id, policy_id)), _): WithRejection<Path<(Uuid, Uuid)>, AppError>,
) -> Result<Json<ar_entity::policy::Model>, AppError> {
    let policy = policy_store::get_policy(policy_set_id.clone(), policy_id.clone(), &db).await?;

    match policy {
        None => Err(AppError::Expected(ExpectedError {
            status_code: StatusCode::NOT_FOUND,
            message: "Can't find policy".to_owned(),
            reason: format!(
                "Unable to find policy with id '{}' and policy set id '{}",
                &policy_id, &policy_set_id
            ),
            metadata: None,
        })),
        Some(p) => Ok(Json(p)),
    }
}

/// Add a new policy to an existing policy set (admin access)
#[utoipa::path(
    post,
    path = "/admin/policy-set/{id}/policy",
    tag = "Policy Management - Admin",
    params(
        ("id" = Uuid, Path, description = "Identifier of the policy set")
    ),
    request_body(
        content = Policy,
        description = "New policy definition to add",
        content_type = "application/json"
    ),
    security(
        ("h2m_bearer_admin" = [])
    ),
    responses(
        (
            status = 200,
            description = "Policy successfully added",
            content_type = "application/json",
            body = ar_entity::policy::Model
        ),
        (
            status = 400,
            description = "Invalid policy or service provider validation failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unable to verify service provider as valid iSHARE party"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
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
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    State(app_state): State<AppState>,
    Json(body): Json<Policy>,
) -> Result<Json<ar_entity::policy::Model>, AppError> {
    for sp in body.target.environment.service_providers.iter() {
        app_state
            .satellite_provider
            .validate_party(app_state.time_provider.now(), sp)
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

    let policy = policy_store::add_policy_to_policy_set(&id, body, &db).await?;

    Ok(Json(policy))
}

/// Replace a policy in a policy set (admin access)
#[utoipa::path(
    put,
    path = "/admin/policy-set/{policy_set_id}/policy/{policy_id}",
    tag = "Policy Management - Admin",
    params(
        ("policy_set_id" = Uuid, Path, description = "Identifier of the policy set"),
        ("policy_id" = Uuid, Path, description = "Identifier of the policy to replace")
    ),
    request_body(
        content = Policy,
        description = "New policy definition",
        content_type = "application/json"
    ),
    security(
        ("h2m_bearer_admin" = [])
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
            description = "Invalid policy or service provider validation failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unable to verify service provider as valid iSHARE party"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
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
    WithRejection(Path((policy_set_id, policy_id)), _): WithRejection<Path<(Uuid, Uuid)>, AppError>,
    State(app_state): State<AppState>,
    Json(body): Json<Policy>,
) -> Result<Json<ar_entity::policy::Model>, AppError> {
    for sp in body.target.environment.service_providers.iter() {
        app_state
            .satellite_provider
            .validate_party(app_state.time_provider.now(), sp)
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

    let policy = policy_store::replace_policy(policy_set_id, policy_id, &body, &db).await?;

    Ok(Json(policy))
}

/// Delete a policy set (admin access)
#[utoipa::path(
    delete,
    path = "/admin/policy-set/{id}",
    tag = "Policy Management - Admin",
    params(
        ("id" = Uuid, Path, description = "Identifier of the policy set to delete")
    ),
    security(
        ("h2m_bearer_admin" = [])
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
            status = 404,
            description = "Policy set not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Policy set not found"))
        )
    )
 )]
async fn delete_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
) -> Result<(), AppError> {
    policy_store::delete_policy_set(&id, &db).await?;

    Ok(())
}

/// Delete a policy from a policy set (admin access)
#[utoipa::path(
    delete,
    path = "/admin/policy-set/{policy_set_id}/policy/{policy_id}",
    tag = "Policy Management - Admin",
    params(
        ("policy_set_id" = Uuid, Path, description = "Identifier of the policy set"),
        ("policy_id" = Uuid, Path, description = "Identifier of the policy to delete")
    ),
    security(
        ("h2m_bearer_admin" = [])
    ),
    responses(
        (
            status = 204,
            description = "Policy successfully deleted"
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        ),
        (
            status = 404,
            description = "Policy or policy set not found",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Policy not found"))
        )
    )
 )]
async fn delete_policy_from_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    WithRejection(Path((_policy_set_id, policy_id)), _): WithRejection<
        Path<(Uuid, Uuid)>,
        AppError,
    >,
) -> Result<(), AppError> {
    policy_store::delete_policy(&policy_id, &db).await?;

    Ok(())
}

/// Get a policy set by ID (admin access)
#[utoipa::path(
    get,
    path = "/admin/policy-set/{id}",
    tag = "Policy Management - Admin",
    params(
        ("id" = Uuid, Path, description = "Identifier of the policy set to retrieve")
    ),
    security(
        ("h2m_bearer_admin" = [])
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
) -> Result<Json<MatchingPolicySetRow>, AppError> {
    let ps = policy_store::get_policy_set_with_policies(&id, &db).await?;

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

#[derive(Serialize, ToSchema)]
struct InsertPolicySetResponse {
    uuid: Uuid,
}

/// Create a new policy set (admin access)
#[utoipa::path(
    post,
    path = "/admin/policy-set",
    tag = "Policy Management - Admin",
    request_body(
        content = policy_store::InsertPolicySetWithPolicies,
        description = "Policy set details and its initial policies",
        content_type = "application/json"
    ),
    security(
        ("h2m_bearer_admin" = [])
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
            description = "Invalid policy set definition",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Invalid policy set format"))
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        )
    )
 )]
async fn insert_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<
        Json<policy_store::InsertPolicySetWithPolicies>,
        AppError,
    >,
) -> Result<Json<InsertPolicySetResponse>, AppError> {
    let policy_set_id = policy_service::insert_policy_set_with_policies_admin(
        app_state.time_provider.now(),
        &body,
        &db,
        app_state.satellite_provider,
    )
    .await?;

    let response = InsertPolicySetResponse {
        uuid: policy_set_id,
    };

    Ok(Json(response))
}

#[derive(Deserialize)]
struct GetPolicySetsQuery {
    access_subject: Option<String>,
    policy_issuer: Option<String>,
    q: Option<String>,
    limit: Option<u32>,
    skip: Option<u32>,
}

/// List all policy sets with optional filtering (admin access)
#[utoipa::path(
    get,
    path = "/admin/policy-set",
    tag = "Policy Management - Admin",
    params(
        ("access_subject" = Option<String>, Query, description = "Filter by access subject"),
        ("policy_issuer" = Option<String>, Query, description = "Filter by policy issuer"),
        ("limit" = Option<u32>, Query, description = "Limit the number of results for pagination"),
        ("skip" = Option<u32>, Query, description = "Skip a number of results for pagination"),
        ("q" = Option<String>, Query, description = "Filter on any match in the policy set"),
    ),
    security(
        ("h2m_bearer_admin" = [])
    ),
    responses(
        (
            status = 200,
            description = "List of policy sets matching the filter criteria",
            content_type = "application/json",
            body = Vec<MatchingPolicySetRow>
        ),
        (
            status = 401,
            description = "Authentication failed",
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized"))
        )
    )
 )]
async fn get_all_policy_sets(
    Query(query): Query<GetPolicySetsQuery>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<PolicySetsWithPagination>, AppError> {
    let policy_sets = policy_store::get_policy_sets_with_policies(
        query.access_subject,
        query.policy_issuer,
        query.q,
        query.skip,
        query.limit,
        &db,
    )
    .await
    .context("Error getting policicy sets")?;

    Ok(Json(policy_sets))
}

#[cfg(test)]
mod test {
    use crate::{
        db::policy::PolicySetsWithPagination,
        fixtures::fixtures::{insert_policy_set_fixture, load_policy_set_fixture},
        routes::admin::InsertPolicySetTemplateResponse,
        services::server_token,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use reqwest::header::AUTHORIZATION;
    use serde_json::json;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;

    use super::super::super::test_helpers::helpers::*;

    #[sqlx::test]
    async fn test_insert_policy_set_template(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "name": "Usual dexspace data consumer stuff",
            "access_subject": "hello",
            "policy_issuer": "hello again",
            "policies": [
              {
                "resource_type": "Fishes",
                "identifiers": ["*"],
                "attributes": ["*"],
                "actions": ["Read", "Delete", "Create", "Edit"],
                "service_providers": ["NL.EORI.LIFEELEC4DMI"],
                "rules": [
                  {
                    "effect": "Permit"
                  }
                ]
              }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set-template")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
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
    async fn test_insert_policy_set_template_with_description(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db);

        let request_body = create_request_body(&json!({
            "name": "Usual dexspace data consumer stuff",
            "access_subject": "hello",
            "policy_issuer": "hello again",
            "description": "this is a nice pt",
            "policies": [
              {
                "resource_type": "Fishes",
                "identifiers": ["*"],
                "attributes": ["*"],
                "actions": ["Read", "Delete", "Create", "Edit"],
                "service_providers": ["NL.EORI.LIFEELEC4DMI"],
                "rules": [
                  {
                    "effect": "Permit"
                  }
                ]
              }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set-template")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
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
    async fn test_insert_delete_policy_set_template(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db.clone());

        let request_body = create_request_body(&json!({
            "name": "Usual dexspace data consumer stuff",
            "access_subject": "hello",
            "policy_issuer": "hello again",
            "policies": [
              {
                "resource_type": "Fishes",
                "identifiers": ["*"],
                "attributes": ["*"],
                "actions": ["Read", "Delete", "Create", "Edit"],
                "service_providers": ["NL.EORI.LIFEELEC4DMI"],
                "rules": [
                  {
                    "effect": "Permit"
                  }
                ]
              }
            ]
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set-template")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body: InsertPolicySetTemplateResponse = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let app = get_test_app(db);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/admin/policy-set-template/{}", body.uuid))
                    .method("DELETE")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
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
    async fn test_get_policy_sets(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 5);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_limit(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?limit=2")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_skip(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;

        let _policy_set4 = load_policy_set_fixture("./fixtures/policy_set4.json");

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?skip=3")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_skip_limit(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;

        let _policy_set4 = load_policy_set_fixture("./fixtures/policy_set4.json");

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?skip=2&limit=2")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_filter_as(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?access_subject=NL.44444")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 4);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_query_as(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set7.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?q=NL.CONSUME")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_query_pi(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set6.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?q=NL.CONSUME")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_filter_resource_type(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set6.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?q=Amazinggg")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_filter_identifier(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set6.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?q=CrazyID")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_filter_att(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set6.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?q=CrazyATT")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_filter_sp(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set6.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?q=CrazyATT")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_policy_sets_filter_pi(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set5.json", &db).await;

        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set?policy_issuer=NL.44444")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: PolicySetsWithPagination = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.data.len(), 1);

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
                "policyIssuer": "sss",
                "licences": [],
                "maxDelegationDepth": 2
        }));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
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
}
