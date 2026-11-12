use axum::{
    extract::{Query, State},
    middleware::from_fn_with_state,
    routing::get,
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    error::AppError,
    middleware::extract_role_middleware,
    services::{
        audit_log::AuditEventWithIssAndSub,
        server_token::{Role, ServerToken},
    },
    AppState,
};

pub fn get_audit_log_routes(server_token: std::sync::Arc<ServerToken>) -> Router<AppState> {
    return Router::new()
        .route("/", get(retrieve_audit_log_entries))
        .layer(from_fn_with_state(
            server_token.clone(),
            extract_role_middleware,
        ));
}

pub fn default_max_results() -> u64 {
    500
}

#[derive(Deserialize)]
struct RetrieveAuditLogEntriesQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    #[serde(rename = "max-results", default = "default_max_results")]
    max_results: u64,
    #[serde(rename = "eventTypes")]
    event_types: Option<String>,
}

async fn retrieve_audit_log_entries(
    Query(query): Query<RetrieveAuditLogEntriesQuery>,
    Extension(db): Extension<DatabaseConnection>,
    State(app_state): State<AppState>,
    Extension(role): Extension<Role>,
) -> Result<Json<Vec<AuditEventWithIssAndSub>>, AppError> {
    let requester_company_id = role.get_company_id();

    let events = crate::services::audit_log::retrieve_events(
        &requester_company_id,
        query.from,
        query.to,
        query.max_results,
        query.event_types,
        app_state.time_provider,
        &app_state.config,
        &db,
    )
    .await?;

    Ok(Json(events))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::fixtures::fixtures::insert_policy_set_fixture;
    use crate::routes::policy_set::InsertPolicySetResponse;
    use crate::services::audit_log::{
        AuditEventWithIssAndSub, EditedType, PolicyAdded, PolicyRemoved, PolicyReplaced,
    };
    use crate::services::server_token;
    use crate::test_helpers::helpers::{create_request_body, get_test_app, init_test_db};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use ishare::v3::delegation_request::{DelegationRequest, DelegationTarget};
    use reqwest::header::AUTHORIZATION;
    use serde_json::json;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_max_results(_pool_options: PgPoolOptions, conn_option: PgConnectOptions) {
        let db = init_test_db(&conn_option).await;

        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        for _ in 0..2000 {
            crate::services::audit_log::log_event(
                chrono::DateTime::parse_from_rfc3339("2025-08-11T09:00:00Z")
                    .unwrap()
                    .to_utc(),
                "".to_owned(),
                crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                    delegation_path: None,
                    previous_steps: None,
                    policy_issuer: "pi".to_owned(),
                    target: DelegationTarget {
                        access_subject: "as".to_owned(),
                    },
                    policy_sets: vec![],
                }),
                Some("included".to_string()),
                None,
                &db,
            )
            .await
            .unwrap();
        }

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 500);

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log?max-results=700")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 700);

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log?max-results=0")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 1);

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log?max-results=1200")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 1000);
    }

    #[sqlx::test]
    async fn test_from_query(_pool_options: PgPoolOptions, conn_option: PgConnectOptions) {
        let db = init_test_db(&conn_option).await;

        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-06-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("not included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-08-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log?from=2025-07-11T09:00:00Z")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 1);
        assert_eq!(audit_log.get(0).unwrap().source, "AR");
    }

    #[sqlx::test]
    async fn test_event_types_query(_pool_options: PgPoolOptions, conn_option: PgConnectOptions) {
        let db = init_test_db(&conn_option).await;

        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-06-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("not included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-06-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("not included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-08-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log?eventTypes=dmi:ar:delegation:request")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 3);
    }

    #[sqlx::test]
    async fn test_iss(_pool_options: PgPoolOptions, conn_option: PgConnectOptions) {
        let db = init_test_db(&conn_option).await;

        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-06-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("not included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.get(0).unwrap().iss, "NL.CONSUME_TOO_MUCH");
    }

    #[sqlx::test]
    async fn test_sub(_pool_options: PgPoolOptions, conn_option: PgConnectOptions) {
        let db = init_test_db(&conn_option).await;

        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-06-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("not included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.get(0).unwrap().sub, "NL.44444");
    }

    #[sqlx::test]
    async fn test_to_query(_pool_options: PgPoolOptions, conn_option: PgConnectOptions) {
        let db = init_test_db(&conn_option).await;

        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-06-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        crate::services::audit_log::log_event(
            chrono::DateTime::parse_from_rfc3339("2025-08-11T09:00:00Z")
                .unwrap()
                .to_utc(),
            "".to_owned(),
            crate::services::audit_log::EventType::DmiDelegationRequest(DelegationRequest {
                delegation_path: None,
                previous_steps: None,
                policy_issuer: "pi".to_owned(),
                target: DelegationTarget {
                    access_subject: "as".to_owned(),
                },
                policy_sets: vec![],
            }),
            Some("not included".to_string()),
            None,
            &db,
        )
        .await
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log?to=2025-07-11T09:00:00Z")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(audit_log.len(), 1);
        assert_eq!(audit_log.get(0).unwrap().source, "AR");
    }

    #[sqlx::test]
    async fn test_delegation_audit_entry(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        let app = get_test_app(db.clone());
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
                                    "context": {
                                        "serviceProvider": "good-company"
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

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:delegation:request")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(context.get("policyIssuer").unwrap(), "NL.24244");
    }

    #[sqlx::test]
    async fn test_policy_set_created_audit_entry(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        let app = get_test_app(db.clone());

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

        let policy_set_response: InsertPolicySetResponse = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:created")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            &policy_set_response.uuid.to_string()
        );
    }

    #[sqlx::test]
    async fn test_admin_policy_set_created_audit_entry(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        let app = get_test_app(db.clone());

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

        let policy_set_response: InsertPolicySetResponse = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:created")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            &policy_set_response.uuid.to_string()
        );
    }

    #[sqlx::test]
    async fn test_remove_policy_from_policy_set_audit_event(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db.clone());

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

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:edited")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();
        let edited_type = serde_json::from_value(serde_json::to_value(&context).unwrap()).unwrap();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881"
        );

        match edited_type {
            EditedType::PolicyRemoved(PolicyRemoved { policy_id }) => {
                assert_eq!(
                    Uuid::parse_str("564f3b46-7127-4c3c-a0b8-2859c01cc9c1").unwrap(),
                    policy_id
                )
            }
            _ => panic!(),
        };

        Ok(())
    }

    #[sqlx::test]
    async fn test_remove_policy_from_policy_set_audit_event_admin(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;

        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy/564f3b46-7127-4c3c-a0b8-2859c01cc9c1")
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

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:edited")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();
        let edited_type = serde_json::from_value(serde_json::to_value(&context).unwrap()).unwrap();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881"
        );

        match edited_type {
            EditedType::PolicyRemoved(PolicyRemoved { policy_id }) => {
                assert_eq!(
                    Uuid::parse_str("564f3b46-7127-4c3c-a0b8-2859c01cc9c1").unwrap(),
                    policy_id
                )
            }
            _ => panic!(),
        };

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_policy_to_policy_set_via_de(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;

        let app = get_test_app(db.clone());

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

        let policy: ar_entity::policy::Model = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:edited")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();
        let edited_type = serde_json::from_value(serde_json::to_value(&context).unwrap()).unwrap();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881",
        );

        match edited_type {
            EditedType::PolicyAdded(PolicyAdded { policy_id }) => {
                assert_eq!(policy.id, policy_id)
            }
            _ => panic!(),
        };

        Ok(())
    }

    #[sqlx::test]
    async fn test_add_policy_to_policy_set_via_de_admin(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set4.json", &db).await;

        let app = get_test_app(db.clone());

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
                    .uri("/admin/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy")
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

        let policy: ar_entity::policy::Model = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:edited")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881"
        );

        let edited_type = serde_json::from_value(serde_json::to_value(&context).unwrap()).unwrap();

        match edited_type {
            EditedType::PolicyAdded(PolicyAdded { policy_id }) => {
                assert_eq!(policy.id, policy_id)
            }
            _ => panic!(),
        };

        Ok(())
    }

    #[sqlx::test]
    async fn test_replace_policy_in_policy_set_audit_event(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db.clone());

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

        let new_policy: ar_entity::policy::Model = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:edited")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881",
        );

        let edited_type = serde_json::from_value(serde_json::to_value(&context).unwrap()).unwrap();

        match edited_type {
            EditedType::PolicyReplaced(PolicyReplaced {
                old_policy_id,
                new_policy_id,
            }) => {
                assert_eq!(
                    old_policy_id,
                    Uuid::parse_str("564f3b46-7127-4c3c-a0b8-2859c01cc9c1").unwrap()
                );
                assert_eq!(new_policy_id, new_policy.id)
            }
            _ => panic!(),
        };

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_policy_set_audit_event(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db.clone());

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

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:deleted")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_policy_set_audit_event_admin(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881")
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

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:deleted")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881"
        );

        Ok(())
    }

    #[sqlx::test]
    async fn test_replace_policy_in_policy_set_audit_event_admin(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        insert_policy_set_fixture("./fixtures/policy_set_audit_log.json", &db).await;
        insert_policy_set_fixture("./fixtures/policy_set1.json", &db).await;

        let app = get_test_app(db.clone());

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
                    .uri("/admin/policy-set/84b7fba4-05f3-4af8-9d84-dde384abe881/policy/564f3b46-7127-4c3c-a0b8-2859c01cc9c1")
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

        let new_policy: ar_entity::policy::Model = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        let audit_log_response = get_test_app(db.clone())
            .oneshot(
                Request::builder()
                    .uri("/audit-log")
                    .method("GET")
                    .header(
                        AUTHORIZATION,
                        server_token::server_token_test_helper::get_human_token_header(
                            Some("NL.44444".to_owned()),
                            Some("lovely-user".to_owned()),
                        ),
                    )
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(audit_log_response.status(), StatusCode::OK);

        let audit_log: Vec<AuditEventWithIssAndSub> = serde_json::from_str(
            std::str::from_utf8(
                &audit_log_response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes(),
            )
            .unwrap(),
        )
        .unwrap();

        let events: Vec<AuditEventWithIssAndSub> = audit_log
            .into_iter()
            .filter(|a| a.event_type == "dmi:ar:policy_set:edited")
            .collect();

        assert_eq!(events.len(), 1);

        let context: HashMap<String, String> = events.get(0).unwrap().context.clone();

        assert_eq!(
            context.get("policy_set_id").unwrap(),
            "84b7fba4-05f3-4af8-9d84-dde384abe881",
        );

        let edited_type = serde_json::from_value(serde_json::to_value(&context).unwrap()).unwrap();

        match edited_type {
            EditedType::PolicyReplaced(PolicyReplaced {
                old_policy_id,
                new_policy_id,
            }) => {
                assert_eq!(
                    old_policy_id,
                    Uuid::parse_str("564f3b46-7127-4c3c-a0b8-2859c01cc9c1").unwrap()
                );
                assert_eq!(new_policy_id, new_policy.id)
            }
            _ => panic!(),
        };

        Ok(())
    }
}
