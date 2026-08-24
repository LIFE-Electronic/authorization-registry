use anyhow::Context;
use serde::{Deserialize, Serialize};
use textnonce::TextNonce;

#[derive(Clone)]
pub struct IdpConnector {
    pub idp_url: String,
    pub client_id: String,
    pub idp_eori: String,
}

#[derive(Serialize)]
pub struct AuthRequestClaims {
    client_id: String,
    scope: String,
    redirect_uri: String,
    response_type: String,
    state: String,
    nonce: String,
    acr_values: String,
    language: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub id_token: String,
}

impl IdpConnector {
    pub fn new(url: String, client_id: String, idp_eori: String) -> Self {
        Self {
            idp_url: url,
            client_id,
            idp_eori,
        }
    }

    pub fn get_realm_url(&self) -> String {
        let idp_url = self.idp_url.trim_end_matches('/');
        return format!("{idp_url}/protocol/openid-connect/auth");
    }

    pub fn get_logout_url(&self, redirect_url: &str) -> anyhow::Result<String> {
        let idp_url = self.idp_url.trim_end_matches('/');
        let mut logout_url =
            reqwest::Url::parse(&format!("{idp_url}/protocol/openid-connect/logout"))
                .context("invalid IDP URL")?;

        logout_url
            .query_pairs_mut()
            .append_pair("post_logout_redirect_uri", redirect_url)
            .append_pair("client_id", &self.client_id);

        Ok(logout_url.to_string())
    }

    pub fn generate_auth_url(&self, client_assertion: &str, state: &str) -> String {
        let idp_url = self.idp_url.trim_end_matches('/');
        let client_id = self.client_id.clone();
        let encoded_state = urlencoding::encode(state);
        let url = format!("{idp_url}/protocol/openid-connect/auth?response_type=code&scope=openid+iSHARE&client_id={client_id}&request={client_assertion}&state={encoded_state}");

        return url;
    }

    pub fn get_auth_request_claims(
        &self,
        server_base_url: &str,
        callback_url: &str,
    ) -> AuthRequestClaims {
        let redirect_uri = self.get_redirect_uri(server_base_url);

        let textnonce = TextNonce::sized_urlsafe(32).unwrap();

        // to-do: generate random nonce
        //let nonce = textnonce.to_string();
        let nonce = textnonce.to_string();
        // once of: urn:http://eidas.europa.eu/LoA/NotNotified/low, urn:http://eidas.europa.eu/LoA/NotNotified/substantial or urn:http://eidas.europa.eu/LoA/NotNotified/high,
        //let acr_values = "";
        //let acr_values = "urn:http://eidas.europa.eu/LoA/NotNotified/substantial";
        let acr_values = "urn:http://eidas.europa.eu/LoA/NotNotified/low";
        let language = "nl";

        return AuthRequestClaims {
            client_id: self.client_id.clone(),
            scope: "openid iSHARE".to_owned(),
            redirect_uri: redirect_uri.to_owned(),
            response_type: "code".to_owned(),
            state: callback_url.to_owned(),
            nonce: nonce.to_owned(),
            acr_values: acr_values.to_owned(),
            language: Some(language.to_owned()),
        };
    }

    fn get_redirect_uri(&self, server_base_url: &str) -> String {
        let uri = format!("{server_base_url}/connect/human/auth/code");
        return uri.to_string();
    }

    pub async fn fetch_token(
        &self,
        server_base_url: &str,
        code: &str,
        client_assertion: &str,
    ) -> anyhow::Result<TokenResponse> {
        let idp_url = self.idp_url.trim_end_matches('/');

        let redirect_uri = self.get_redirect_uri(server_base_url);

        let form_data = vec![
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("client_id", &self.client_id),
            ("code", code),
            ("client_assertion", client_assertion),
            (
                "client_assertion_type",
                "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
            ),
        ];

        let response = reqwest::Client::new()
            .post(format!("{idp_url}/protocol/openid-connect/token"))
            .form(&form_data)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .context("error fetching token")?;

        if !response.status().is_success() {
            anyhow::bail!("error response from idp: {:?}", response);
        }

        let token_response = response
            .json::<TokenResponse>()
            .await
            .context("Error decoding token response")?;

        return Ok(token_response);
    }
}

#[cfg(test)]
mod tests {
    use super::IdpConnector;

    fn connector(idp_url: &str) -> IdpConnector {
        IdpConnector::new(idp_url.to_owned(), "client".to_owned(), "idp".to_owned())
    }

    #[test]
    fn realm_url_does_not_contain_a_double_slash_in_the_path() {
        let connector = connector("https://idp.example/realms/dmi/");

        assert_eq!(
            connector.get_realm_url(),
            "https://idp.example/realms/dmi/protocol/openid-connect/auth"
        );
    }

    #[test]
    fn logout_url_is_derived_from_backend_idp_config() {
        let connector = connector("https://idp.example/realms/dmi/");

        let logout_url = connector
            .get_logout_url("http://localhost:5173/member?tab=active")
            .unwrap();
        let parsed = reqwest::Url::parse(&logout_url).unwrap();

        assert_eq!(parsed.path(), "/realms/dmi/protocol/openid-connect/logout");
        assert_eq!(
            parsed
                .query_pairs()
                .find(|(key, _)| key == "post_logout_redirect_uri")
                .map(|(_, value)| value.into_owned()),
            Some("http://localhost:5173/member?tab=active".to_owned())
        );
        assert_eq!(
            parsed
                .query_pairs()
                .find(|(key, _)| key == "client_id")
                .map(|(_, value)| value.into_owned()),
            Some("client".to_owned())
        );
    }
}
