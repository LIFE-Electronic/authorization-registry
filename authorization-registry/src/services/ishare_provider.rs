use anyhow::Context;
use reqwest::StatusCode;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use axum::async_trait;
use ishare::{
    delegation_evidence::DelegationEvidenceContainer,
    ishare::{Capabilities, CertificatesOrSpor, PartyInfo, ValidatePartyError, ISHARE},
};
use std::sync::Arc;
use tokio::sync::RwLock;

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

#[derive(Serialize)]
pub struct OAuthRequestForm {
    pub response_type: String,
    pub scope: String,
    pub request: String,
    pub client_id: String,
    pub state: String,
}

#[async_trait]
pub trait SatelliteProvider: Send + Sync {
    async fn get_satellite_token(&self) -> anyhow::Result<String>;

    fn handle_h2m_redirect_url_request(
        &self,
        server_url: &str,
        redirect_url: &str,
    ) -> anyhow::Result<String>;

    fn get_h2m_redirect_base_url(&self) -> String;

    async fn get_h2m_redirect_form(
        &self,
        server_url: &str,
        redirect_url: &str,
    ) -> anyhow::Result<OAuthRequestForm>;

    async fn handle_h2m_auth_callback(
        &self,
        server_url: &str,
        code: &str,
    ) -> Result<(String, UserOption), AppError>;

    async fn handle_m2m_authentication(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        client_id: &str,
        grant_type: &str,
        client_assertion: &str,
        client_assertion_type: &str,
        scope: &str,
        validate_certificate: bool,
    ) -> Result<String, AppError>;

    async fn validate_party(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        eori: &str,
    ) -> Result<PartyInfo, ValidatePartyError>;

    fn create_delegation_token(
        &self,
        audience: &str,
        de_container: &DelegationEvidenceContainer,
    ) -> anyhow::Result<String>;

    fn create_capabilities_token(
        &self,
        audience: &str,
        capabilities: &Capabilities,
    ) -> anyhow::Result<String>;

    fn handle_previous_step_client_assertion(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        requestor_company_id: &str,
        client_assertion: &str,
        policy_issuer: &str,
        access_subject: &str,
    ) -> bool;
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
            .create_client_assertion_with_extra_claims(audience.to_owned(), de_container)
            .context("Error creating delegation token")
    }

    fn create_capabilities_token(
        &self,
        audience: &str,
        capabilities: &Capabilities,
    ) -> anyhow::Result<String> {
        self.ishare
            .create_client_assertion_with_extra_claims(audience.to_string(), capabilities)
            .context("Error creating delegation token")
    }

    async fn get_satellite_token(&self) -> anyhow::Result<String> {
        let now = chrono::Utc::now().timestamp();
        let mut write_lock = self.satellite_token_cache.write().await;

        if write_lock.is_invalid(now) {
            tracing::info!("satellite access token has expired. fetching new one");

            let client_assertion = self
                .ishare
                .create_client_assertion(self.ishare.satellite_eori.clone())?;
            let token_response = self
                .ishare
                .get_satelite_access_token(&client_assertion)
                .await
                .context("Error retrieving satelite access token")?;

            write_lock.update(
                token_response.access_token.clone(),
                token_response.expires_in + now,
            );

            Ok(token_response.access_token)
        } else {
            tracing::info!("retrieving satellite access token from cache");
            Ok(write_lock.access_token.clone())
        }
    }

    async fn validate_party(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        eori: &str,
    ) -> Result<PartyInfo, ValidatePartyError> {
        let token = self
            .get_satellite_token()
            .await
            .context("Error getting sattelite token")?;

        let party_info = self
            .ishare
            .validate_party(now, eori, &token)
            .await
            .context(format!(
                "error validating company '{}' is ishare party",
                eori
            ))?;

        return Ok(party_info);
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
                self.idp_connector.idp_eori.clone(),
                auth_claims,
            )
            .context("Error creating client assertion")?;

        let url = self
            .idp_connector
            .generate_auth_url(&client_assertion, redirect_url);

        Ok(url)
    }

    fn get_h2m_redirect_base_url(&self) -> String {
        return self.idp_connector.get_realm_url();
    }

    async fn get_h2m_redirect_form(
        &self,
        server_url: &str,
        redirect_url: &str,
    ) -> anyhow::Result<OAuthRequestForm> {
        let auth_claims = self
            .idp_connector
            .get_auth_request_claims(server_url, redirect_url);

        let idp_eori = self.idp_connector.idp_eori.clone();
        let now = chrono::Utc::now();

        let idp_info = self
            .validate_party(now, idp_eori.as_str())
            .await
            .context("Error getting party info IDP")?;

        let idp_cert = match idp_info.certificates_or_spor {
            CertificatesOrSpor::Spor(_) => {
                anyhow::bail!("Error: IDP should have certificate instead of spor");
            }
            CertificatesOrSpor::Certificates(certificates) => certificates
                .into_iter()
                .next()
                .context("No certificate found for IDP")?,
        };

        let encrypted_client_assertion = self
            .ishare
            .create_client_assertion_with_extra_claims_encrypted(
                self.idp_connector.idp_eori.clone(),
                auth_claims,
                &idp_cert,
            )
            .context("Error creating encyrpted client assertion")?;

        let oauth_params = OAuthRequestForm {
            response_type: "code".to_string(),
            scope: "openid ishare".to_string(),
            request: encrypted_client_assertion,
            client_id: self.idp_connector.client_id.clone(),
            state: redirect_url.to_string(),
        };

        Ok(oauth_params)
    }

    fn handle_previous_step_client_assertion(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        requestor_company_id: &str,
        client_assertion: &str,
        policy_issuer: &str,
        access_subject: &str,
    ) -> bool {
        let policy_issuer_access = match self.ishare.decode_token(
            now,
            client_assertion,
            policy_issuer,
            Some(requestor_company_id),
        ) {
            Err(e) => {
                tracing::info!("no acces for policy issuer via previous step: {}", e);
                false
            }
            Ok(_) => {
                tracing::info!("access granted for policy issuer via previous step");
                true
            }
        };

        let access_subject_access = match self.ishare.decode_token(
            now,
            client_assertion,
            access_subject,
            Some(requestor_company_id),
        ) {
            Err(e) => {
                tracing::info!("no acces for access subject via previous step: {}", e);
                false
            }
            Ok(_) => {
                tracing::info!("access granted for access subject via previous step");
                true
            }
        };

        policy_issuer_access || access_subject_access
    }

    async fn handle_h2m_auth_callback(
        &self,
        server_url: &str,
        code: &str,
    ) -> Result<(String, UserOption), AppError> {
        let client_assertion = self
            .ishare
            .create_client_assertion(self.idp_connector.idp_eori.clone())
            .context("Error creating client assertion")?;

        let response = self
            .idp_connector
            .fetch_token(&server_url, code, &client_assertion)
            .await
            .context("Error fetching token from idp")?;

        tracing::info!("got id token {}", response.id_token.clone());

        let decoded_id_token = self
            .ishare
            .decode_token_custom_claims::<IdTokenClaims>(&response.id_token, None)
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
        now: chrono::DateTime<chrono::Utc>,
        client_id: &str,
        grant_type: &str,
        client_assertion: &str,
        client_assertion_type: &str,
        scope: &str,
        validate_certificate: bool,
    ) -> Result<String, AppError> {
        tracing::info!(
            "handeling machine 2 machine authentication for client_id: {}",
            client_id
        );

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

        match self
            .ishare
            .validate_token(&client_assertion.to_string())
            .context("Error validating client assertion")
        {
            Ok(true) => {}
            Ok(false) => {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "client assertion is invalid".to_owned(),
                    reason: "invalid client assertion".to_owned(),
                    metadata: None,
                }));
            }
            Err(e) => {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "client assertion is invalid".to_owned(),
                    reason: format!("error while validating client assertion: {}", e),
                    metadata: None,
                }));
            }
        }

        // probably need to be more explicit here in case the token has expired etc
        let client_assertion_token = self
            .ishare
            .decode_token(now, &client_assertion, client_id, None)
            .map_err(|e| {
                return AppError::Expected(ExpectedError {
                    status_code: StatusCode::BAD_REQUEST,
                    message: "client assertion is invalid".to_owned(),
                    reason: format!("{:?}", e),
                    metadata: None,
                });
            })?;

        let token = self.get_satellite_token().await?;

        let party_info = self
            .ishare
            .validate_party(now, &client_id.to_string(), &token)
            .await
            .context(format!("error validating ishare party '{}'", &client_id))?;

        if validate_certificate {
            if !self
                .ishare
                .validate_party_certificate(&client_assertion_token, &party_info)
                .context(format!(
                    "Error validating party certificate for ishare party: '{}'",
                    &client_id
                ))?
            {
                return Err(AppError::Expected(ExpectedError {
                    status_code: StatusCode::UNAUTHORIZED,
                    message: "x5c header does not match any of the certificates from parties endpoint at the iSHARE satelite".to_owned(),
                    reason: "The client assertion x5c does not match any of the valid tokens from /parties".to_owned(),
                    metadata: None
                }));
            }
        }

        let company_id =
            company_store::insert_if_not_exists(&client_id, &party_info.party_name, &self.db)
                .await
                .context("Error inserting company into db")?;

        return Ok(company_id);
    }
}
