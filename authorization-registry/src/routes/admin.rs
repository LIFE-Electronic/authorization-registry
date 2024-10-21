use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::extract::WithRejection;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::policy as policy_service;
use crate::{
    db::policy::{self as policy_store, MatchingPolicySetRow},
    error::ExpectedError,
};
use crate::{error::AppError, AppState};
use crate::{
    middleware::{auth_role_middleware, extract_human_middleware, extract_role_middleware},
    services::server_token::ServerToken,
};

pub fn get_admin_routes(server_token: Arc<ServerToken>) -> Router<AppState> {
    return Router::new()
        .route(
            "/policy-set",
            post(insert_policy_set).get(get_all_policy_sets),
        )
        .route("/policy-set/:id", get(get_policy_set))
        .layer(from_fn_with_state(
            vec!["dexspace_admin".to_owned()],
            auth_role_middleware,
        ))
        .layer(from_fn(extract_human_middleware))
        .layer(from_fn_with_state(server_token, extract_role_middleware));
}

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

#[derive(Serialize)]
struct InsertPolicySetResponse {
    uuid: Uuid,
}

async fn insert_policy_set(
    Extension(db): Extension<DatabaseConnection>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<
        Json<policy_store::InsertPolicySetWithPolicies>,
        AppError,
    >,
) -> Result<Json<InsertPolicySetResponse>, AppError> {
    let policy_set_id = policy_service::insert_policy_set_with_policies_admin(
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
}

async fn get_all_policy_sets(
    Query(query): Query<GetPolicySetsQuery>,
    Extension(db): Extension<DatabaseConnection>,
) -> Result<Json<Vec<MatchingPolicySetRow>>, AppError> {
    let policy_sets =
        policy_store::get_policy_sets_with_policies(query.access_subject, query.policy_issuer, &db)
            .await
            .context("Error getting policicy sets")?;

    Ok(Json(policy_sets))
}

#[cfg(test)]
mod test {
    use crate::{
        db::policy::MatchingPolicySetRow, fixtures::fixtures::insert_policy_set_fixture,
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

        let body: Vec<MatchingPolicySetRow> = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.len(), 5);

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

        let body: Vec<MatchingPolicySetRow> = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.len(), 4);

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

        let body: Vec<MatchingPolicySetRow> = serde_json::from_str(
            std::str::from_utf8(&response.into_body().collect().await.unwrap().to_bytes()).unwrap(),
        )
        .unwrap();

        assert_eq!(body.len(), 1);

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
                "licences": ["ISHARE.0001"],
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
