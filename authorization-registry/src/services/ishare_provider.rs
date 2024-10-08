use anyhow::Context;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use axum::async_trait;
use ishare::{
    delegation_evidence::DelegationEvidenceContainer,
    ishare::{PartyInfo, ValidatePartyError, ISHARE},
};
use std::sync::{Arc, RwLock};

use crate::{
    db::{company as company_store, user::insert_if_not_exists},
    error::{AppError, ExpectedError},
    token_cache::TokenCache,
};

use super::{idp_connector::IdpConnector, server_token::UserOption};

#[derive(Deserialize)]
struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Deserialize)]
struct IdTokenClaims {
    pub realm_access: RealmAccess,
    pub company_id: String,
    pub company_name: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[async_trait]
pub trait SatelliteProvider: Send + Sync {
    async fn get_satellite_token(&self) -> anyhow::Result<String>;

    fn handle_h2m_redirect_url_request(
        &self,
        server_url: &str,
        redirect_url: &str,
    ) -> anyhow::Result<String>;

    async fn handle_h2m_auth_callback(
        &self,
        server_url: &str,
        code: &str,
    ) -> Result<(String, UserOption), AppError>;

    async fn handle_m2m_authentication(
        &self,
        client_id: &str,
        grant_type: &str,
        client_assertion: &str,
        client_assertion_type: &str,
        scope: &str,
    ) -> Result<String, AppError>;

    async fn validate_party(&self, eori: &str) -> Result<PartyInfo, ValidatePartyError>;

    fn create_delegation_token(
        &self,
        audience: &str,
        de_container: &DelegationEvidenceContainer,
    ) -> anyhow::Result<String>;
}

#[derive(Clone)]
pub struct ISHAREProvider {
    ishare: Arc<ISHARE>,
    db: DatabaseConnection,
    idp_connector: IdpConnector,
    satellite_token_cache: Arc<RwLock<TokenCache>>,
}

impl ISHAREProvider {
    pub fn new(
        ishare: Arc<ISHARE>,
        db: &DatabaseConnection,
        idp_connector: &IdpConnector,
    ) -> ISHAREProvider {
        return ISHAREProvider {
            ishare: ishare.clone(),
            db: db.clone(),
            idp_connector: idp_connector.clone(),
            satellite_token_cache: TokenCache::new(),
        };
    }
}

#[async_trait]
impl SatelliteProvider for ISHAREProvider {
    fn create_delegation_token(
        &self,
        audience: &str,
        de_container: &DelegationEvidenceContainer,
    ) -> anyhow::Result<String> {
        self.ishare
            .create_client_assertion_with_extra_claims(Some(audience.to_owned()), de_container)
            .context("Error creating delegation token")
    }

    async fn get_satellite_token(&self) -> anyhow::Result<String> {
        let now = chrono::Utc::now().timestamp();
        let is_expired = self
            .satellite_token_cache
            .read()
            .map_or(true, |tc| tc.is_invalid(now));

        let token = if is_expired {
            tracing::debug!("satellite access token has expired. fetching new one");
            let client_assertion = self
                .ishare
                .create_client_assertion(Some(self.ishare.sattelite_eori.clone()))?;
            let token_response = self
                .ishare
                .get_satelite_access_token(&client_assertion)
                .await
                .context("Error retrieving satelite access token")?;

            let now = chrono::Utc::now().timestamp();
            let mut mutable_cache = self.satellite_token_cache.write().unwrap();

            mutable_cache.update(token_response.access_token, token_response.expires_in + now);

            mutable_cache.access_token.clone()
        } else {
            tracing::debug!("retrieving satellite access token from cache");
            self.satellite_token_cache
                .read()
                .unwrap()
                .access_token
                .clone()
        };

        Ok(token)
    }

    async fn validate_party(&self, eori: &str) -> Result<PartyInfo, ValidatePartyError> {
        let token = self
            .get_satellite_token()
            .await
            .context("Error getting sattelite token")?;

        let party_token = self
            .ishare
            .validate_party(eori, &token)
            .await
            .context(format!(
                "error validating company '{}' is ishare party",
                eori
            ))?;

        return Ok(party_token.claims.party_info);
    }

    fn handle_h2m_redirect_url_request(
        &self,
        server_url: &str,
        redirect_url: &str,
    ) -> anyhow::Result<String> {
        let auth_claims = self
            .idp_connector
            .get_auth_request_claims(server_url, redirect_url);
        let client_assertion = self
            .ishare
            .create_client_assertion_with_extra_claims(
                Some(self.idp_connector.idp_eori.clone()),
                auth_claims,
            )
            .context("Error creating client assertion")?;

        let url = self
            .idp_connector
            .generate_auth_url(&client_assertion, redirect_url);

        Ok(url)
    }

    async fn handle_h2m_auth_callback(
        &self,
        server_url: &str,
        code: &str,
    ) -> Result<(String, UserOption), AppError> {
        let client_assertion = self
            .ishare
            .create_client_assertion(Some(self.idp_connector.idp_eori.clone()))
            .context("Error creating client assertion")?;

        let response = self
            .idp_connector
            .fetch_token(&server_url, code, &client_assertion)
            .await
            .context("Error fetching token from idp")?;

        let decoded_id_token = self
            .ishare
            .decode_token_custom_claims::<IdTokenClaims>(&response.id_token)
            .context("Error decoding id_token")?;

        let company_id = company_store::insert_if_not_exists(
            &decoded_id_token.claims.extra.company_id,
            &decoded_id_token.claims.extra.company_name,
            &self.db,
        )
        .await
        .context("Error inserting company into db")?;

        let fullname = format!(
            "{} {}",
            decoded_id_token.claims.extra.first_name, decoded_id_token.claims.extra.last_name
        );
        let user_id = insert_if_not_exists(
            decoded_id_token.claims.ishare_claims.sub,
            decoded_id_token.claims.extra.email,
            fullname,
            company_id.clone(),
            self.idp_connector.idp_eori.clone(),
            self.idp_connector.idp_url.clone(),
            &self.db,
        )
        .await
        .context("Error inserting user into db")?;

        let realm_access_roles = decoded_id_token.claims.extra.realm_access.roles;

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
        client_id: &str,
        grant_type: &str,
        client_assertion: &str,
        client_assertion_type: &str,
        scope: &str,
    ) -> Result<String, AppError> {
        match ishare::ishare::validate_request_arguments(grant_type, client_assertion_type, scope) {
            Err(message) => {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message,
                    reason: "Invalid iSHARE Request arguments!".to_owned(),
                    metadata: None,
                }));
            }
            _ => {}
        }

        if !self
            .ishare
            .validate_token(&client_assertion.to_string())
            .context("Error validating client assertion")?
        {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::UNAUTHORIZED,
                message: "client assertion is invalid".to_owned(),
                reason: "invalid client assertion".to_owned(),
                metadata: None,
            }));
        };

        // probably need to be more explicit here in case the token has expired etc
        let client_assertion_token = self.ishare.decode_token(&client_assertion).map_err(|e| {
            return AppError::Expected(ExpectedError {
                status_code: StatusCode::UNAUTHORIZED,
                message: "client assertion is invalid".to_owned(),
                reason: format!("{:?}", e),
                metadata: None,
            });
        })?;

        let token = self.get_satellite_token().await?;

        let party_token = self
            .ishare
            .validate_party(&client_id.to_string(), &token)
            .await
            .context(format!("error validating ishare party '{}'", &client_id))?;

        if !self
            .ishare
            .validate_party_certificate(&client_assertion_token, &party_token)
            .context("Error validating party certificate")?
        {
            return Err(AppError::Expected(ExpectedError {
                status_code: StatusCode::UNAUTHORIZED,
                message: "x5c header does not match any of the certificates from parties endpoint at the iSHARE satelite".to_owned(),
                  reason: "The client assertion x5c does not match any of the valid tokens from /parties".to_owned(),
                  metadata: None
            }));
        }

        let company_id = company_store::insert_if_not_exists(
            &client_id,
            &party_token.claims.party_info.party_name,
            &self.db,
        )
        .await
        .context("Error inserting company into db")?;

        return Ok(company_id);
    }
}
