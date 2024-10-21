use crate::services::idp_connector::IdpConnector;
use crate::services::ishare_provider::{ISHAREProvider, SatelliteProvider};
use crate::services::server_token::ServerToken;

use axum::async_trait;
use axum::http::HeaderValue;
use axum::Extension;
use axum::{extract::FromRef, Router};
use clap::Parser;
use ishare::ishare::ISHARE;
use routes::admin::get_admin_routes;
use routes::connect::get_connect_routes;
use routes::delegation::get_delegation_routes;
use routes::policy_set::get_policy_set_routes;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod db;
mod error;
mod fixtures;
mod middleware;
mod routes;
mod services;
mod test_helpers;
mod token_cache;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./.config.json")]
    config_path: String,
}

#[async_trait]
pub trait TimeProvider: Send + Sync {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

struct RealTimeProvider;

impl RealTimeProvider {
    fn new() -> Self {
        Self {}
    }
}

impl TimeProvider for RealTimeProvider {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

pub struct AppConfig {
    pub deploy_route: String,
}

#[derive(Clone)]
pub struct AppState {
    server_token: Arc<ServerToken>,
    satellite_provider: Arc<dyn SatelliteProvider>,
    time_provider: Arc<dyn TimeProvider>,
    de_expiry_seconds: i64,
    config: Arc<AppConfig>,
}

impl FromRef<AppState> for Arc<ServerToken> {
    fn from_ref(app_state: &AppState) -> Arc<ServerToken> {
        app_state.server_token.clone()
    }
}

pub fn get_app(db: DatabaseConnection, app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods(AllowMethods::any())
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_headers(AllowHeaders::any());

    let connect_routes = get_connect_routes();
    let admin_routes = get_admin_routes(app_state.server_token.clone());
    let delegation_routes = get_delegation_routes(app_state.server_token.clone());
    let policy_set_routes = get_policy_set_routes(app_state.server_token.clone());

    let app = Router::new()
        .nest("/connect", connect_routes)
        .nest("/admin", admin_routes)
        .nest("/delegation", delegation_routes)
        .nest("/policy-set", policy_set_routes)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<axum::extract::MatchedPath>()
                    .map(axum::extract::MatchedPath::as_str);

                let span = tracing::info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                );
                span.in_scope(|| {
                    tracing::info!(
                        "Incoming request [method = {}, path = \"{}\"]",
                        request.method(),
                        request.uri()
                    );
                });

                span
            }),
        )
        .layer(Extension(db))
        .layer(cors)
        .with_state(app_state);

    return app;
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = config::read_config(args.config_path);

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db = Database::connect(config.database_url.clone())
        .await
        .unwrap();

    let server_token = ServerToken::new(config.jwt_secret, config.jwt_expiry_seconds);
    let ishare = Arc::new(
        ISHARE::new(
            config.client_cert_path,
            config.client_cert_pass,
            config.satellite_url,
            Some(config.ishare_ca_path),
            config.client_eori.clone(),
            config.satellite_eori,
        )
        .unwrap(),
    );
    let idp_connector = IdpConnector::new(config.idp_url, config.client_eori, config.idp_eori);
    let sat_provider = ISHAREProvider::new(ishare.clone(), &db, &idp_connector);
    let time_provider = RealTimeProvider::new();
    let app_state = AppState {
        server_token: Arc::new(server_token),
        satellite_provider: Arc::new(sat_provider),
        time_provider: Arc::new(time_provider),
        de_expiry_seconds: config.de_expiry_seconds,
        config: Arc::new(AppConfig {
            deploy_route: config.deploy_route.clone(),
        }),
    };

    let app = get_app(db, app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
