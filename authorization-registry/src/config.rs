use serde::Deserialize;

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

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
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
}

pub fn read_config(path: String) -> Config {
    let file_content =
        std::fs::read(&path).expect(&format!("Failed to read config file: '{}'", &path));
    let config = serde_json::from_slice(&file_content).expect("unable to parse config");

    return config;
}
