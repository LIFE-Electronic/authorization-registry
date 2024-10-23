use crate::error::AppError;
use crate::{services::server_token::ServerToken, AppState};
use anyhow::Context;
use axum::extract::Query;
use axum::response::Redirect;
use axum::Json;
use axum::{
    extract::Host,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Form, Router,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn get_connect_routes() -> Router<AppState> {
    let router = Router::new()
        .route("/machine/token", post(get_machine_token))
        .route("/human/auth", get(get_auth))
        .route("/human/auth/code", get(get_auth_callback));

    return router;
}

#[derive(Serialize)]
struct AuthClaims {
    client_id: String,
    scope: String,
    redirect_uri: String,
    response_type: String,
    state: String,
}

fn get_server_base_url(
    headers: HeaderMap,
    host: String,
    deploy_route: &str,
) -> Result<String, AppError> {
    let scheme = match headers.get("X-Forwarded-Proto") {
        None => "http://".to_string(),
        Some(s) => {
            s.to_str()
                .context("error converting to string")?
                .to_string()
                + "://"
        }
    };
    let server_base_url = [scheme, host, deploy_route.to_owned()].join("");

    return Ok(server_base_url);
}

#[derive(Deserialize, Serialize)]
struct AuthQuery {
    redirect_uri: String,
    state: Option<String>,
}

async fn get_auth(
    State(app_state): State<AppState>,
    Host(host): Host,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> Result<Redirect, AppError> {
    let server_base_url = get_server_base_url(headers, host, &app_state.config.deploy_route)?;
    let state = serde_json::to_string(&query).map_err(|e| {
        return AppError::Unexpected(anyhow::anyhow!("error serialising auth query: {}", e));
    })?;

    let redirect_url = app_state
        .satellite_provider
        .handle_h2m_redirect_url_request(&server_base_url, &state)
        .map_err(|err| {
            tracing::error!("error handling h2m redirect request");
            err
        })?;

    return Ok(Redirect::to(&redirect_url));
}

#[derive(Deserialize)]
struct AuthCallbackQuery {
    code: String,
    state: String,
}

async fn get_auth_callback(
    State(state): State<AppState>,
    State(server_token): State<Arc<ServerToken>>,
    Host(host): Host,
    headers: HeaderMap,
    query: Query<AuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    let server_base_url = get_server_base_url(headers, host, &state.config.deploy_route)?;

    let (company_id, user_option) = state
        .satellite_provider
        .handle_h2m_auth_callback(&server_base_url, &query.code)
        .await
        .map_err(|err| {
            tracing::error!("error handling h2m auth callback");
            err
        })?;

    let action_token = server_token
        .create_token(company_id, Some(user_option))
        .map_err(|err| {
            tracing::error!("error creating access token");
            err
        })?;

    let auth_query = serde_json::from_str::<AuthQuery>(&query.state).unwrap();
    let mut redirect_url = Url::parse(&auth_query.redirect_uri).unwrap();
    let mut query: Vec<(std::borrow::Cow<str>, std::borrow::Cow<str>)> =
        redirect_url.query_pairs().collect();

    query.push((
        std::borrow::Cow::Borrowed("token"),
        std::borrow::Cow::Borrowed(&action_token),
    ));

    if let Some(state) = &auth_query.state {
        query.push((
            std::borrow::Cow::Borrowed("state"),
            std::borrow::Cow::Borrowed(state),
        ));
    }

    let query_strings: Vec<String> = query.iter().map(|c| format!("{}={}", c.0, c.1)).collect();
    let query_string = query_strings.join("&");
    redirect_url.set_query(Some(&query_string));

    return Ok(Redirect::to(redirect_url.as_str()));
}

#[derive(Serialize, Debug)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Deserialize, Debug)]
struct TokenRequest {
    grant_type: String,
    client_assertion_type: String,
    client_id: String,
    client_assertion: String,
    scope: String,
}

async fn get_machine_token(
    State(server_token): State<Arc<ServerToken>>,
    State(state): State<AppState>,
    Form(body): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let company_id = state
        .satellite_provider
        .handle_m2m_authentication(
            &body.client_id,
            &body.grant_type,
            &body.client_assertion,
            &body.client_assertion_type,
            &body.scope,
        )
        .await?;

    let service_access_token = server_token.create_token(company_id, None)?;

    Ok(Json(TokenResponse {
        access_token: service_access_token,
        expires_in: server_token.jwt_expiry_seconds,
        token_type: "Bearer".to_owned(),
    }))
}
