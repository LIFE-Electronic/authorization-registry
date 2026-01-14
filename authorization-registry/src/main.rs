use crate::config::FrontendConfig;
use crate::routes::audit_log::get_audit_log_routes;
use crate::services::idp_connector::IdpConnector;
use crate::services::ishare_provider::{ISHAREProvider, SatelliteProvider};
use crate::services::server_token::ServerToken;
use ar_migration::{Migrator, MigratorTrait};

use axum::async_trait;
use axum::Extension;
use axum::extract::MatchedPath;
use axum::{extract::FromRef, Router};
use clap::Parser;
use ishare::ishare::ISHARE;
use routes::admin::get_admin_routes;
use routes::capabilities::get_capabilities_routes;
use routes::connect::get_connect_routes;
use routes::delegation::get_delegation_routes;
use routes::policy_set::get_policy_set_routes;
use routes::policy_set_template::get_policy_set_template_routes;
use sea_orm::Database;
use sea_orm::DatabaseConnection;
use seed::apply_seeds;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod db;
mod error;
mod fixtures;
mod middleware;
mod routes;
mod seed;
mod services;
mod test_helpers;
mod token_cache;
mod utils;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
                "Bearer Token for Authorize Header",
            ))),
        );
        components.add_security_scheme(
            "h2m_bearer_admin",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
                "Bearer Token for Authorize Header",
            ))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Authorization Registry",
        description = "Authorization Registry API that conforms to the iSHARE framework to manage policy storage and authorization delegation.

Authentication is required for most endpoints. Non-admin routes can be accessed with either a Machine-to-Machine (M2M) token or a Human-to-Machine (H2M) token, obtainable via the authentication endpoints (/connect/machine/token for M2M, /connect/human/auth for H2M). Admin routes require an H2M token with the dexspace_admin role.

Policy management endpoints allow participants to create and manage authorization policy sets that define what rights are delegated to which service consumers. Policies follow the iSHARE Delegation Evidence format to ensure interoperability across the iSHARE network.

Admin routes provide additional capabilities for managing policies across all participants, while regular routes are scoped to the authenticated party's own policies.",
        version = "1.0",
        contact(name="WolperTec B.V."),
        license(name="License: GPL v3")
    ),
    modifiers(&SecurityAddon),
    paths(
        routes::delegation::post_delegation,
        routes::capabilities::get_capabilities,
        routes::connect::get_machine_token,
        routes::connect::get_auth,
        routes::connect::get_auth_callback,
        routes::policy_set::get_all_policy_sets,
        routes::policy_set::get_policy_set,
        routes::policy_set::insert_policy_set,
        routes::policy_set::delete_policy_set,
        routes::policy_set::add_policy_to_policy_set,
        routes::policy_set::delete_policy_from_policy_set,
        routes::policy_set::replace_policy_in_policy_set,
        routes::admin::get_policy,
        routes::admin::add_policy_to_policy_set,
        routes::admin::replace_policy_in_policy_set,
        routes::admin::delete_policy_set,
        routes::admin::delete_policy_from_policy_set,
        routes::admin::get_policy_set,
        routes::admin::insert_policy_set,
        routes::admin::get_all_policy_sets,
        routes::admin::insert_policy_set_template,
        routes::admin::delete_policy_set_template,
        routes::policy_set_template::get_policy_set_template,
        routes::policy_set_template::get_policy_set_templates,
    )
)]
struct ApiDoc;

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

#[derive(Debug)]
pub struct AppConfig {
    pub deploy_route: String,
    pub client_eori: String,
    pub validate_m2m_certificate: bool,
    pub delegation_allows_service_providers: bool,
    pub frontend: FrontendConfig,
    pub service_name: String,
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

pub fn get_app(db: DatabaseConnection, app_state: AppState, disable_cors_check: bool) -> Router {
    let connect_routes = get_connect_routes();
    let admin_routes = get_admin_routes(app_state.server_token.clone());
    let delegation_routes = get_delegation_routes(app_state.server_token.clone());
    let policy_set_routes = get_policy_set_routes(app_state.server_token.clone());
    let capabilities_routes = get_capabilities_routes();
    let policy_set_template_routes = get_policy_set_template_routes(app_state.server_token.clone());
    let audit_log_routes = get_audit_log_routes(app_state.server_token.clone());
    let config_routes = routes::config::get_config_routes();

    let app = Router::new()
        .nest("/connect", connect_routes)
        .nest("/admin", admin_routes)
        .nest("/delegation", delegation_routes)
        .nest("/policy-set", policy_set_routes)
        .nest("/capabilities", capabilities_routes)
        .nest("/policy-set-template", policy_set_template_routes)
        .nest("/audit-log", audit_log_routes.clone())
        .nest("/audit-log/", audit_log_routes)
        .nest("/config", config_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(control_plane_logging::make_http_span)
                .on_request(control_plane_logging::on_http_request)
                .on_response(control_plane_logging::get_on_http_response()),
        )
        .layer(Extension(db))
        .with_state(app_state);

    if disable_cors_check {
        return app.layer(
            CorsLayer::new()
                .allow_methods(AllowMethods::any())
                .allow_origin(AllowOrigin::any())
                .allow_headers(AllowHeaders::any()),
        );
    } else {
        return app;
    };
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = config::read_config(args.config_path);

    control_plane_logging::init_tracing();

    tracing::info!("Deploy route: {}", config.deploy_route);

    let db = Database::connect(config.database_url.clone())
        .await
        .unwrap();

    Migrator::up(&db, None).await.unwrap();
    apply_seeds(&db, &config).await;

    let server_token = ServerToken::new(config.jwt_secret, config.jwt_expiry_seconds);
    let ishare = Arc::new(
        ISHARE::new(
            config.client_cert_path,
            config.client_cert_pass,
            config.satellite_url,
            Some(config.ishare_ca_path),
            config.client_eori.clone(),
            config.satellite_eori,
            config.dataspace_config.clone(),
        )
        .unwrap(),
    );
    let idp_connector =
        IdpConnector::new(config.idp_url, config.client_eori.clone(), config.idp_eori);
    let sat_provider = ISHAREProvider::new(ishare.clone(), &db, &idp_connector);
    let time_provider = RealTimeProvider::new();
    let app_state = AppState {
        server_token: Arc::new(server_token),
        satellite_provider: Arc::new(sat_provider),
        time_provider: Arc::new(time_provider),
        de_expiry_seconds: config.de_expiry_seconds,
        config: Arc::new(AppConfig {
            deploy_route: config.deploy_route.clone(),
            client_eori: config.client_eori.clone(),
            validate_m2m_certificate: config.validate_m2m_certificate,
            delegation_allows_service_providers: config.delegation_allows_service_providers,
            frontend: config.frontend,
            service_name: config.service_name,
        }),
    };

    tracing::info!("application config --- [{:?}]", app_state.config);

    let app = get_app(db, app_state, config.disable_cors_check);

    let listener = tokio::net::TcpListener::bind(config.listen_address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
