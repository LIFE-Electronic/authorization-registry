use anyhow::Context;
use axum::{
    extract::{Host, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    error::{AppError, ExpectedError},
    services::ishare_provider::{
        Capabilities, CapabilitiesInfo, SupportedFeature, SupportedFeatures, SupportedVersion,
    },
    AppState,
};
use utoipa::ToSchema;

pub fn get_capabilities_routes() -> Router<AppState> {
    return Router::new().route("/", get(get_capabilities));
}

pub fn create_capabilities(party_id: &str, api_url: &str, show_private: bool) -> Capabilities {
    let mut supported_features: Vec<SupportedFeatures> = vec![SupportedFeatures::Public(vec![
        SupportedFeature {
            id: "ebb696ab-bda7-44a9-8cec-382183d58d9d".to_owned(),
            feature: "access token".to_owned(),
            url: format!("{}{}", api_url, "/connect/machine/token"),
            description: "retrieve machine access token for M2M authentication".to_owned(),
            token_endpoint: None,
        },
        SupportedFeature {
            id: "ebb696ab-bda7-44a9-8cec-382183d58d9d".to_owned(),
            feature: "human access token".to_owned(),
            url: format!("{}{}", api_url, "/connect/machine/token"),
            description: "retrieve human access token for H2M authentication".to_owned(),
            token_endpoint: None,
        },
        SupportedFeature {
            id: "d7d27d71-2755-4eea-bb97-bfa5ce8addef".to_owned(),
            feature: "capabilities".to_owned(),
            url: format!("{}/capabilities", api_url),
            description: "retrieve capabilities".to_owned(),
            token_endpoint: Some(format!("{}/connect/machine/token", api_url)),
        },
    ])];

    if show_private {
        supported_features.push(SupportedFeatures::Private(vec![SupportedFeature {
            id: "aaf5162b-82f2-4bf2-9eaa-e01b380e7ec3".to_owned(),
            url: format!("{}{}", api_url, "/delegation"),
            feature: "iSHARE delegation request".to_owned(),
            description: "issue iSHARE delegation evidence based on your delegation request"
                .to_owned(),
            token_endpoint: Some(format!("{}{}", api_url, "/connect/machine/token")),
        }]));
    }

    return Capabilities {
        capabilities_info: CapabilitiesInfo {
            party_id: party_id.to_owned(),
            ishare_roles: vec!["AuthorizationRegistry".to_owned()],
            supported_versions: vec![SupportedVersion {
                version: "1.7".to_owned(),
                supported_features,
            }],
        },
    };
}

#[derive(Serialize, ToSchema)]
struct CapabilitiesResponse {
    capabilities_token: String,
}

pub fn extract_bearer_token(header_map: &HeaderMap) -> Result<Option<String>, AppError> {
    let auth_header = match header_map.get("Authorization") {
        Some(header) => header,
        None => return Ok(None),
    };

    match auth_header
        .to_str()
        .context("Removing bearer prefix from auth header")?
        .strip_prefix("Bearer ")
    {
        Some(token) => {
            return Ok(Some(token.to_owned()));
        }
        None => {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::BAD_REQUEST,
                message: "Missing bearer prefix in Authorization header".to_owned(),
                reason: "Missing bearer prefix in Authorization header".to_owned(),
                metadata: None,
            }));
        }
    };
}

/// Retrieve iSHARE capabilities
#[utoipa::path(
    get,
    path = "/capabilities",
    tag = "Capabilities",
    responses(
        (
            status = 200,
            description = "Authorization Registry capabilities",
            content_type = "application/json",
            body = CapabilitiesResponse
        ),
    )
 )]
#[axum_macros::debug_handler]
async fn get_capabilities(
    header_map: HeaderMap,
    Host(host): Host,
    State(app_state): State<AppState>,
) -> Result<Json<CapabilitiesResponse>, AppError> {
    let (show_private, audience) = match extract_bearer_token(&header_map) {
        Err(e) => return Err(e),
        Ok(Some(raw_token)) => {
            let token = app_state.server_token.decode_token(&raw_token)?;

            (true, token.claims.role.get_company_id())
        }
        Ok(None) => (false, "public".to_owned()),
    };

    let scheme = match header_map.get("X-Forwarded-Proto") {
        None => "http://".to_string(),
        Some(s) => {
            s.to_str()
                .context("error converting to string")?
                .to_string()
                + "://"
        }
    };

    let api_url = [scheme, host, app_state.config.deploy_route.clone()].join("");
    let capabilities = create_capabilities(&app_state.config.client_eori, &api_url, show_private);

    let capabilities_token = app_state
        .satellite_provider
        .create_capabilities_token(&audience, &capabilities)?;

    let response = CapabilitiesResponse { capabilities_token };

    return Ok(Json(response));
}

#[cfg(test)]
mod test {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };

    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;

    use super::super::super::test_helpers::helpers::*;

    #[sqlx::test]
    async fn auth_header_not_bearer_plus_value(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/capabilities")
                    .header("Host", "Example.com")
                    .header("Authorization", "Chicken token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_public_capabilities(
        _pool_options: PgPoolOptions,
        conn_option: PgConnectOptions,
    ) -> sqlx::Result<()> {
        let db = init_test_db(&conn_option).await;
        let app = get_test_app(db);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/capabilities")
                    .header("Host", "Example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        Ok(())
    }
}
