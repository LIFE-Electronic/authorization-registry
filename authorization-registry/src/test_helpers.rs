#[cfg(test)]
pub mod helpers {
    use ar_migration::{Migrator, MigratorTrait};
    use axum::body::Body;
    use axum::{async_trait, Router};
    use ishare::delegation_evidence::DelegationEvidenceContainer;
    use ishare::ishare::{Adherence, PartyInfo, ValidatePartyError};
    use sea_orm::{Database, DatabaseConnection};
    use serde_json::Value;
    use sqlx::{postgres::PgConnectOptions, ConnectOptions};
    use std::sync::Arc;

    use crate::error::AppError;
    use crate::get_app;
    use crate::services::ishare_provider::SatelliteProvider;
    use crate::services::server_token::{server_token_test_helper, UserOption};
    use crate::AppState;
    use crate::TimeProvider;

    struct FakeTimeProvider;

    impl FakeTimeProvider {
        fn new() -> Self {
            Self {}
        }
    }

    impl TimeProvider for FakeTimeProvider {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            return chrono::DateTime::from_timestamp(1715247205, 0).unwrap();
        }
    }

    pub async fn init_test_db(conn_option: &PgConnectOptions) -> DatabaseConnection {
        let opts = conn_option
            .to_owned()
            .ssl_mode(sqlx::postgres::PgSslMode::Allow);
        let url = opts.to_url_lossy();
        let database = Database::connect(url).await.unwrap();

        Migrator::up(&database, None).await.unwrap();
        crate::fixtures::fixtures::apply(&database).await;

        return database;
    }

    pub fn create_request_body(json: &Value) -> Body {
        let body = serde_json::to_string(json).unwrap();
        let mut bytes: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut bytes, &body).unwrap();
        return Body::new(body);
    }

    pub fn get_test_app(db: DatabaseConnection) -> Router {
        let sat_provider = TestSatelliteProvider {};
        let server_token = server_token_test_helper::get_test_service();

        let app_state = AppState {
            server_token: Arc::new(server_token),
            satellite_provider: Arc::new(sat_provider.clone()),
            time_provider: Arc::new(FakeTimeProvider::new()),
            de_expiry_seconds: 3600,
            config: Arc::new(crate::AppConfig {
                deploy_route: "".to_owned(),
            }),
        };
        let app = get_app(db, app_state);

        return app;
    }

    #[derive(Clone)]
    pub struct TestSatelliteProvider {}

    #[async_trait]
    impl SatelliteProvider for TestSatelliteProvider {
        async fn get_satellite_token(&self) -> anyhow::Result<String> {
            return Ok("token".to_string());
        }

        fn create_delegation_token(
            &self,
            _audience: &str,
            _de_container: &DelegationEvidenceContainer,
        ) -> anyhow::Result<String> {
            Ok("delegation token".to_owned())
        }

        async fn validate_party(&self, eori: &str) -> Result<PartyInfo, ValidatePartyError> {
            return Ok(PartyInfo {
                adherence: Adherence {
                    status: "Active".to_string(),
                },
                party_id: eori.to_string(),
                party_name: "cool party".to_string(),
                certificates: vec![],
            });
        }

        fn handle_h2m_redirect_url_request(
            &self,
            _server_url: &str,
            _redirect_url: &str,
        ) -> anyhow::Result<String> {
            let url = "a_url".to_string();
            Ok(url)
        }

        async fn handle_h2m_auth_callback(
            &self,
            _server_url: &str,
            _code: &str,
        ) -> Result<(String, UserOption), AppError> {
            let company_id = "A_company".to_string();
            let user_id = uuid::Uuid::new_v4().to_string();
            let realm_access_roles = vec!["role1".to_string(), "role2".to_string()];

            return Ok((
                company_id,
                UserOption {
                    user_id,
                    realm_access_roles,
                },
            ));
        }

        async fn handle_m2m_authentication(
            &self,
            _client_id: &str,
            _grant_type: &str,
            _client_assertion: &str,
            _client_assertion_type: &str,
            _scope: &str,
        ) -> Result<String, AppError> {
            return Ok("A_company".to_string());
        }
    }
}
