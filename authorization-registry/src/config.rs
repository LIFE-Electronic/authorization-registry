use ishare::ishare::AllowedDataspaces;
use serde::{Deserialize, Serialize};

fn default_listen_address() -> String {
    "0.0.0.0:4000".to_string()
}

fn default_jwt_expiry_seconds() -> u64 {
    3600
}

fn default_de_expiry_seconds() -> i64 {
    3600
}

fn default_deploy_route() -> String {
    "/api".to_owned()
}

fn default_disable_cors_check() -> bool {
    true
}

fn default_validate_m2m_certificate() -> bool {
    true
}

fn default_delegation_allows_service_providers() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NavigationConfig {
    pub passport: String,
    pub catalogue: String,
    pub clearing: String,
    pub datastation: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressConfig {
    pub name: String,
    pub address_content: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContactConfig {
    pub address: AddressConfig,
    pub tax_number: String,
    pub email: String,
    pub phone_number: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneralConfig {
    pub become_member: String,
    pub faq: String,
    pub about: String,
    pub support: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocialsConfig {
    pub linkedin: String,
    pub x: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FooterConfig {
    pub navigation: NavigationConfig,
    pub contact: ContactConfig,
    pub general: GeneralConfig,
    pub socials: SocialsConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FrontendConfig {
    pub footer: FooterConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub frontend: FrontendConfig,
    pub client_eori: String,
    pub idp_url: String,
    pub idp_eori: String,
    pub client_cert_path: String,
    pub client_cert_pass: String,
    pub satellite_url: String,
    pub ishare_ca_path: String,
    pub satellite_eori: String,
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry_seconds")]
    pub jwt_expiry_seconds: u64,
    pub database_url: String,
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    #[serde(default = "default_de_expiry_seconds")]
    pub de_expiry_seconds: i64,
    #[serde(default = "default_deploy_route")]
    pub deploy_route: String,
    pub seed_folder: Option<String>,
    #[serde(default = "default_disable_cors_check")]
    pub disable_cors_check: bool,
    #[serde(default = "default_validate_m2m_certificate")]
    pub validate_m2m_certificate: bool,
    #[serde(default = "default_delegation_allows_service_providers")]
    pub delegation_allows_service_providers: bool,
    pub dataspace_config: Option<AllowedDataspaces>,
}

pub fn read_config(path: String) -> Config {
    let file_content =
        std::fs::read(&path).expect(&format!("Failed to read config file: '{}'", &path));
    let config = serde_json::from_slice(&file_content).expect("unable to parse config");

    return config;
}
