use serde::{Deserialize, Serialize};
use url::Url;
use std::collections::HashMap;
use rand::Rng;
use base64::Engine;
use sha2::{Sha256, Digest};
use reqwest::Client;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::config::{get_client_id,get_client_secret};


lazy_static! {
    static ref CODE_VERIFIER_STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
}

impl Default for GoogleOAuthConfig {
    
    fn default() -> Self {
        Self {
            client_id: get_client_id().to_string(),
            client_secret: get_client_secret().to_string(),
            redirect_uri: "http://127.0.0.1:0/callback".to_string(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub id_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
}

pub struct GoogleOAuth {
    pub config: GoogleOAuthConfig,
    client: Client,
}

impl GoogleOAuth {
    pub fn new(config: GoogleOAuthConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn set_redirect_uri(&mut self, redirect_uri: String) {
        self.config.redirect_uri = redirect_uri;
    }

    pub fn generate_auth_url(&self) -> Result<(String, String), String> {
        let mut rng = rand::thread_rng();
        
        // Generate code verifier (43-128 characters, base64url allowed characters)
        let code_verifier: String = (0..64)
            .map(|_| {
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
                let idx = rng.gen_range(0..chars.len());
                chars.chars().nth(idx).unwrap()
            })
            .collect();

        // Generate code challenge
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(hasher.finalize());

        // Generate state parameter for CSRF protection
        let state: String = (0..16)
            .map(|_| format!("{:02x}", rng.gen::<u8>()))
            .collect();

        let mut url = Url::parse(&self.config.auth_url)
            .map_err(|e| format!("Failed to parse auth URL: {}", e))?;

        let scopes = vec!["openid", "email", "profile"].join(" ");

        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &scopes)
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");

        // Store code verifier with state as key
        {
            let mut store = CODE_VERIFIER_STORE.lock().unwrap();
            store.insert(state.clone(), code_verifier);
        }

        Ok((url.to_string(), state))
    }

    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        state: &str,
    ) -> Result<AuthResponse, String> {
        // Retrieve the stored code verifier
        let code_verifier = {
            let store = CODE_VERIFIER_STORE.lock().unwrap();
            store
                .get(state)
                .cloned()
                .ok_or_else(|| "Invalid state parameter".to_string())?
        };

        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("code_verifier", &code_verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        let response = self.client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("OAuth error: {} - {}", status, error_text));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Clean up the stored code verifier
        {
            let mut store = CODE_VERIFIER_STORE.lock().unwrap();
            store.remove(state);
        }

        Ok(auth_response)
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo, String> {
        let response = self.client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("Failed to get user info: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Failed to get user info: {} - {}", status, text));
        }

        let user_info: GoogleUserInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))?;

        Ok(user_info)
    }
}
