use anyhow::Context;
use axum::{
    extract::{Host, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use ishare::v3::{Capabilities, CapabilitiesInfo, Service, ServiceVersion};
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    error::{AppError, ExpectedError},
    AppState,
};
use utoipa::ToSchema;

pub fn get_capabilities_routes() -> Router<AppState> {
    return Router::new().route("/", get(get_capabilities));
}

fn service(
    identifier: &str,
    title: &str,
    description: &str,
    endpoint_url: String,
    token_endpoint: Option<String>,
    service_type: &str,
) -> Service {
    Service {
        identifier: identifier.to_owned(),
        title: title.to_owned(),
        description: Some(description.to_owned()),
        endpoint_description: None,
        endpoint_url,
        token_endpoint,
        status: "active".to_owned(),
        service_type: service_type.to_owned(),
        version: ServiceVersion {
            complies_with_framework_versions: Some(vec!["3.0".to_owned()]),
            complies_with_dataspace_versions: None,
            capability_version: "1.0".to_owned(),
        },
        methods: None,
        auth_registry: None,
    }
}

pub fn create_capabilities(api_url: &str, show_private: bool) -> Capabilities {
    let token_endpoint = format!("{}/connect/machine/token", api_url);
    let public_services = vec![
        service(
            "request-oauth-token",
            "access token",
            "retrieve machine access token for M2M authentication",
            token_endpoint.clone(),
            None,
            "framework-defined",
        ),
        service(
            "human-access-token",
            "human access token",
            "retrieve human access token for H2M authentication",
            format!("{}/connect/human/auth", api_url),
            None,
            "self-defined",
        ),
        service(
            "get-capabilities-token",
            "capabilities",
            "retrieve capabilities",
            format!("{}/capabilities", api_url),
            Some(token_endpoint.clone()),
            "framework-defined",
        ),
        service(
            "igds-auditlog",
            "igds-auditlog",
            "InformationGrid DataSharing auditlog",
            format!("{}/audit-log", api_url),
            Some(token_endpoint.clone()),
            "self-defined",
        ),
    ];

    let restricted_services = show_private.then(|| {
        vec![service(
            "request-delegation-evidence",
            "iSHARE delegation request",
            "issue iSHARE delegation evidence based on your delegation request",
            format!("{}/delegation", api_url),
            Some(token_endpoint.clone()),
            "framework-defined",
        )]
    });

    return Capabilities {
        capabilities_info: CapabilitiesInfo {
            public_services,
            restricted_services,
        },
    };
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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
    let capabilities = create_capabilities(&api_url, show_private);

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

    use http_body_util::BodyExt;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use tower::ServiceExt;

    use super::super::super::test_helpers::helpers::*;

    #[test]
    fn human_access_token_points_to_human_auth() {
        let capabilities = super::create_capabilities("http://ar", false);

        let human = capabilities
            .capabilities_info
            .public_services
            .iter()
            .find(|s| s.identifier == "human-access-token")
            .unwrap();

        assert_eq!(human.endpoint_url, "http://ar/connect/human/auth");
    }

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

        let body: serde_json::Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();

        assert_eq!(
            body,
            serde_json::json!({ "capabilitiesToken": "capabilities token" })
        );

        Ok(())
    }
}
