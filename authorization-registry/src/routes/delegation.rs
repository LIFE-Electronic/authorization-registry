use anyhow::Context;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::middleware::from_fn_with_state;
use axum::response::{IntoResponse, Response};
use axum::{extract::Extension, routing::post, Json, Router};
use axum_extra::extract::WithRejection;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorResponse, ExpectedError};
use crate::middleware::extract_role_middleware;
use crate::services::delegation as delegation_service;
use crate::services::server_token::{Role, ServerToken};
use crate::AppState;
use ishare::delegation_request::DelegationRequestContainer;

pub fn get_delegation_routes(server_token: std::sync::Arc<ServerToken>) -> Router<AppState> {
    Router::new()
        .route("/", post(post_delegation))
        .layer(from_fn_with_state(server_token, extract_role_middleware))
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
struct DelegationResponse {
    delegation_token: String,
}

/// Obtain Delegation Evidence
#[utoipa::path(
    post,
    path = "/delegation",
    tag = "Delegation",
    request_body(
        description="Delegation Request",
        content((DelegationRequestContainer))
    ),
    security(
        ("bearer" = [])
    ),
    responses(
        (
            status = 200,
            description = "OK. JSON with delegation evidence", 
            content_type = "application/json",
            body = DelegationResponse,
        ),
        (
            status = 400,
            description = "Malformed request", 
            content_type = "application/text/plain; charset=utf-8",
            body = String,
        ),
        (
            status = 401,
            description = "Unauthorized", 
            content_type = "application/json",
            example = json!(ErrorResponse::new("Unauthorized")),
        ),
    )
)]
#[axum_macros::debug_handler]
async fn post_delegation(
    headers: HeaderMap,
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    app_state: State<AppState>,
    body: WithRejection<Json<DelegationRequestContainer>, AppError>,
) -> Result<Response, AppError> {
    match app_state
        .satellite_provider
        .validate_party(
            app_state.time_provider.now(),
            &body.delegation_request.policy_issuer,
        )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: "policy issuer is not valid iSHARE party".to_owned(),
                reason: format!(
                    "Unable to verify policy issuer: '{} as valid iSHARE party | {}",
                    &body.delegation_request.policy_issuer, e
                ),
                metadata: None,
            }))
        }
    }

    match app_state
        .satellite_provider
        .validate_party(
            app_state.time_provider.now(),
            &body.delegation_request.target.access_subject,
        )
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: "access subject is not valid iSHARE party".to_owned(),
                reason: format!(
                    "Unable to verify access subject: '{} as valid iSHARE party | {}",
                    &body.delegation_request.target.access_subject, e
                ),
                metadata: None,
            }))
        }
    }

    tracing::info!("checking if requester is policy issuer or access subject");
    if &role.get_company_id() != &body.delegation_request.policy_issuer
        && &role.get_company_id() != &body.delegation_request.target.access_subject
    {
        tracing::info!("requester is not policy issuer or access subject");
        tracing::info!("checking if previous steps gives access");

        if let Some(previous_steps) = &body.previous_steps {
            let previous_step_client_assertion = match previous_steps.get(0) {
                None => {
                    tracing::info!("previous steps is empty");
                    return Err(AppError::Expected(ExpectedError { status_code: StatusCode::BAD_REQUEST,
                        message: format!("only policy issuer and access subject are permitted"),
                        reason: format!("company: {} is not policy issuer or access subject and is not permitted to perfrom delegation request", &role.get_company_id()), metadata: None }));
                }
                Some(ps) => ps,
            };

            if !app_state
                .satellite_provider
                .handle_previous_step_client_assertion(
                    app_state.time_provider.now(),
                    &role.get_company_id(),
                    &previous_step_client_assertion,
                    &body.delegation_request.policy_issuer,
                    &body.delegation_request.target.access_subject,
                )
            {
                return Err(AppError::Expected(ExpectedError { status_code: StatusCode::BAD_REQUEST,
                        message: format!("only policy issuer and access subject are permitted. access via previous step failed."),
                        reason: format!("no access granted via previous step for requester: {}", &role.get_company_id()), metadata: None }));
            }
        } else {
            tracing::info!("no previous steps found in request");
            return Err(AppError::Expected(ExpectedError { status_code: StatusCode::BAD_REQUEST,
                message: format!("only policy issuer and access subject are permitted"),
                reason: format!("company: {} is not policy issuer or access subject and is not permitted to perfrom delegation request", &role.get_company_id()), metadata: None }));
        }
    }

    for ps in &body.delegation_request.policy_sets {
        for policy in &ps.policies {
            if policy.target.resource.resource_type == "*" {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "resource type cannot be '*'".to_owned(),
                    reason: "'*' used as resource type in policy set".to_owned(),
                    metadata: None,
                }));
            }

            if policy.target.resource.identifiers.len() == 0 {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "identifiers is empty'".to_owned(),
                    reason: "identifiers in policy set cannot be an empty array".to_owned(),
                    metadata: None,
                }));
            }

            if policy.target.resource.attributes.len() == 0 {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "attributes is empty".to_owned(),
                    reason: "attributes in policy set cannot be an empty array".to_owned(),
                    metadata: None,
                }));
            }
        }
    }

    let delegation_evidence_container = delegation_service::create_delegation_evidence(
        &body.delegation_request,
        app_state.time_provider.clone(),
        app_state.de_expiry_seconds,
        &db,
    )
    .await?;

    let token = app_state
        .satellite_provider
        .create_delegation_token(&role.get_company_id(), &delegation_evidence_container)
        .context("Error creating delegation token")?;

    let response = match headers.get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"application/json") => Json(delegation_evidence_container).into_response(),
        _ => Json(DelegationResponse {
            delegation_token: token,
        })
        .into_response(),
    };

    return Ok(response);
}

#[cfg(test)]
mod test {
    use ishare::delegation_evidence::DelegationEvidenceContainer;

    use crate::fixtures::fixtures::insert_policy_set_fixture;
    use crate::services::server_token;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use reqwest::header::AUTHORIZATION;
    use serde_json;
    use serde_json::json;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;

    use super::super::super::test_helpers::helpers::*;

    #[sqlx::test]
    async fn test_delegation_evidence_not_as_or_pi(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "TestResource",
                                        "identifiers": ["test4"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("OtherCompany".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "TestResource",
                                        "identifiers": ["test4"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: DelegationEvidenceContainer = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.delegation_evidence.policy_sets.len() > 0, true);

        for ps in body.delegation_evidence.policy_sets.iter() {
            for p in ps.policies.iter() {
                assert_eq!(p.rules.get(0).unwrap().effect, "Permit")
            }
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_retrieve_jwt(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "test-iden",
                                        "identifiers": ["test4"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
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
    async fn test_delegation_evidence_wildcard(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set3.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "test-iden",
                                        "identifiers": ["test4"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: DelegationEvidenceContainer = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.delegation_evidence.policy_sets.len() > 0, true);

        for ps in body.delegation_evidence.policy_sets.iter() {
            for p in ps.policies.iter() {
                assert_eq!(p.rules.get(0).unwrap().effect, "Permit")
            }
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_unallowed_action(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "test-iden",
                                        "identifiers": ["test4"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete", "Fish"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: DelegationEvidenceContainer = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.delegation_evidence.policy_sets.len() == 1, true);
        assert_eq!(
            body.delegation_evidence
                .policy_sets
                .get(0)
                .unwrap()
                .policies
                .get(0)
                .unwrap()
                .rules
                .get(0)
                .unwrap()
                .effect
                == "Deny",
            true
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_specfic_identifier_match(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "test-iden",
                                        "identifiers": ["specific"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: DelegationEvidenceContainer = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.delegation_evidence.policy_sets.len() > 0, true);

        for ps in body.delegation_evidence.policy_sets.iter() {
            for p in ps.policies.iter() {
                assert_eq!(p.rules.get(0).unwrap().effect, "Deny")
            }
        }

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_resource_type_start(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "*",
                                        "identifiers": ["specific"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_no_identifier(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "something",
                                        "identifiers": [],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_no_attributes(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "something",
                                        "identifiers": ["zainger"],
                                        "attributes": []
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(None, None),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delegation_evidence_specfic_identifier_no_match(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set2.json", &db).await;

        let app = get_test_app(db);
        let request_body = create_request_body(&json!({
            "delegationRequest": {
                "policyIssuer": "NL.24244",
                "target": {
                    "accessSubject": "NL.44444"
                },
                "policySets": [
                    {
                        "policies": [
                            {
                                "target": {
                                    "resource": {
                                        "type": "test-iden",
                                        "identifiers": ["something else"],
                                        "attributes": ["zingers"]
                                    },
                                    "actions": ["Read", "Delete"],
                                    "environment": {
                                        "serviceProviders": ["good-company"]
                                    }
                                },
                                "rules": [
                                    {
                                        "effect": "Permit"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/delegation")
                    .method("POST")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            None,
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(Body::new(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: DelegationEvidenceContainer = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.delegation_evidence.policy_sets.len() > 0, true);

        for ps in body.delegation_evidence.policy_sets.iter() {
            for p in ps.policies.iter() {
                assert_eq!(p.rules.get(0).unwrap().effect, "Permit")
            }
        }

        Ok(())
    }
}
