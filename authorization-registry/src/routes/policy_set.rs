use ar_entity::delegation_evidence::Policy;
use axum::extract::Path;
use axum::routing::delete;
use axum::{
    extract::State, middleware::from_fn_with_state, routing::post, Extension, Json, Router,
};
use axum_extra::extract::WithRejection;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::policy::MatchingPolicySetRow;
use crate::error::ExpectedError;
use crate::services::policy::{self as policy_service};
use crate::{db::policy as policy_store, services::server_token::Role};
use crate::{error::AppError, AppState};
use crate::{middleware::extract_role_middleware, services::server_token::ServerToken};

pub fn get_policy_set_routes(server_token: Arc<ServerToken>) -> Router<AppState> {
    return Router::new()
        .route("/", post(insert_policy_set))
        .route("/:id", delete(delete_policy_set).get(get_policy_set))
        .route("/:id/policy", post(add_policy_to_policy_set))
        .route(
            "/:id/policy/:policy_id",
            delete(delete_policy_from_policy_set),
        )
        .layer(from_fn_with_state(server_token, extract_role_middleware));
}

async fn delete_policy_from_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path((policy_set_id, policy_id)), _): WithRejection<Path<(Uuid, Uuid)>, AppError>,
    State(app_state): State<AppState>,
) -> Result<(), AppError> {
    policy_service::remove_policy_from_policy_set(
        &role.get_company_id(),
        &policy_set_id,
        &policy_id,
        app_state.time_provider,
        &db,
    )
    .await?;

    Ok(())
}

async fn get_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    Extension(role): Extension<Role>,
    State(app_state): State<AppState>,
) -> Result<Json<MatchingPolicySetRow>, AppError> {
    let ps = policy_service::get_policy_set_with_policies(
        &role.get_company_id(),
        &id,
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

#[axum_macros::debug_handler]
async fn add_policy_to_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    State(app_state): State<AppState>,
    Json(body): Json<Policy>,
) -> Result<Json<ar_entity::policy::Model>, AppError> {
    let policy = policy_service::add_policy_to_policy_set(
        &role.get_company_id(),
        &id,
        body,
        app_state.time_provider,
        &db,
    )
    .await?;

    Ok(Json(policy))
}

async fn delete_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, AppError>,
    State(app_state): State<AppState>,
) -> Result<(), AppError> {
    policy_service::delete_policy_set(&role.get_company_id(), &id, app_state.time_provider, &db)
        .await?;

    Ok(())
}

#[derive(Serialize)]
struct InsertPolicySetResponse {
    uuid: Uuid,
}

async fn insert_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    Extension(role): Extension<Role>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<
        Json<policy_store::InsertPolicySetWithPolicies>,
        AppError,
    >,
) -> Result<Json<InsertPolicySetResponse>, AppError> {
    let policy_set_id = policy_service::insert_policy_set_with_policies(
        &role.get_company_id(),
        &body,
        &db,
        app_state.time_provider.clone(),
        app_state.satellite_provider.clone(),
    )
    .await?;

    let response = InsertPolicySetResponse {
        uuid: policy_set_id,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod test {
    use crate::{fixtures::fixtures::insert_policy_set_fixture, services::server_token};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use reqwest::header::AUTHORIZATION;
    use serde_json::json;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;

    use super::super::super::test_helpers::helpers::*;

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
                "licences": ["ISHARE.0001"],
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
                "licences": ["ISHARE.0001"],
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
                "licences": ["ISHARE.0001"],
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
