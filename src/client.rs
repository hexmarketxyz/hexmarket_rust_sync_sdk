//! HexMarket API client (synchronous).

use serde::de::DeserializeOwned;

use crate::auth::{self, ApiCredentials, L2Headers};
use crate::error::HexSdkError;

/// Configuration for the HexMarket client.
#[derive(Debug, Clone)]
pub struct HexClientConfig {
    /// API base URL, e.g. `https://api.hexmarket.xyz` or `http://localhost:8080`.
    pub api_url: String,
}

/// Synchronous HexMarket API client.
///
/// # Example
///
/// ```no_run
/// use hexmarket_sdk_sync::{HexClient, HexClientConfig, ApiCredentials};
///
/// let client = HexClient::new(HexClientConfig {
///     api_url: "https://api.hexmarket.xyz".into(),
/// });
///
/// // Public endpoints (no auth needed)
/// let events = client.list_events(&Default::default()).unwrap();
///
/// // Authenticated endpoints
/// client.set_credentials(
///     "your-solana-pubkey",
///     ApiCredentials {
///         api_key: "your-api-key".into(),
///         secret: "your-base64url-secret".into(),
///         passphrase: "your-passphrase".into(),
///     },
/// );
///
/// let balance = client.get_balance().unwrap();
/// ```
pub struct HexClient {
    pub(crate) agent: ureq::Agent,
    pub(crate) base_url: String,
    credentials: std::sync::RwLock<Option<(String, ApiCredentials)>>,
}

impl HexClient {
    /// Create a new client.
    pub fn new(config: HexClientConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(10))
            .timeout_read(std::time::Duration::from_secs(30))
            .build();

        Self {
            agent,
            base_url: config.api_url.trim_end_matches('/').to_string(),
            credentials: std::sync::RwLock::new(None),
        }
    }

    /// Set API credentials for L2-authenticated endpoints.
    pub fn set_credentials(&self, pubkey: &str, creds: ApiCredentials) {
        *self.credentials.write().unwrap() = Some((pubkey.to_string(), creds));
    }

    /// Clear stored credentials.
    pub fn clear_credentials(&self) {
        *self.credentials.write().unwrap() = None;
    }

    /// Build a full URL from a path.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Build L2 auth headers for the current credentials.
    pub(crate) fn l2_headers(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<L2Headers, HexSdkError> {
        let guard = self.credentials.read().unwrap();
        let (pubkey, creds) = guard.as_ref().ok_or(HexSdkError::MissingCredentials)?;
        auth::build_l2_headers(creds, pubkey, method, path, body)
    }

    /// Get the stored public key.
    pub(crate) fn require_pubkey(&self) -> Result<String, HexSdkError> {
        let guard = self.credentials.read().unwrap();
        let (pubkey, _) = guard.as_ref().ok_or(HexSdkError::MissingCredentials)?;
        Ok(pubkey.clone())
    }

    /// Send a GET request and parse the JSON response.
    pub(crate) fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, HexSdkError> {
        let resp = self.agent.get(url).call()?;
        resp.into_json::<T>().map_err(|e| HexSdkError::InvalidResponse(e.to_string()))
    }

    /// Send an authenticated GET request and parse the JSON response.
    pub(crate) fn get_auth<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, HexSdkError> {
        let headers = self.l2_headers("GET", path, None)?;
        let resp = headers
            .apply(self.agent.get(&self.url(path)))
            .call()?;
        resp.into_json::<T>().map_err(|e| HexSdkError::InvalidResponse(e.to_string()))
    }

    /// Send an authenticated POST request with JSON body and parse response.
    pub(crate) fn post_auth<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, HexSdkError> {
        let headers = self.l2_headers("POST", path, None)?;
        let resp = headers
            .apply(self.agent.post(&self.url(path)))
            .send_json(body)?;
        resp.into_json::<T>().map_err(|e| HexSdkError::InvalidResponse(e.to_string()))
    }

    /// Send an authenticated PUT request with JSON body and parse response.
    pub(crate) fn put_auth<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, HexSdkError> {
        let headers = self.l2_headers("PUT", path, None)?;
        let resp = headers
            .apply(self.agent.put(&self.url(path)))
            .send_json(body)?;
        resp.into_json::<T>().map_err(|e| HexSdkError::InvalidResponse(e.to_string()))
    }

    /// Send an authenticated DELETE request and parse response.
    pub(crate) fn delete_auth<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, HexSdkError> {
        let headers = self.l2_headers("DELETE", path, None)?;
        let resp = headers
            .apply(self.agent.delete(&self.url(path)))
            .call()?;
        resp.into_json::<T>().map_err(|e| HexSdkError::InvalidResponse(e.to_string()))
    }

    /// Send an authenticated DELETE request with JSON body and parse response.
    pub(crate) fn delete_auth_with_body<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, HexSdkError> {
        let headers = self.l2_headers("DELETE", path, None)?;
        let resp = headers
            .apply(self.agent.delete(&self.url(path)))
            .send_json(body)?;
        resp.into_json::<T>().map_err(|e| HexSdkError::InvalidResponse(e.to_string()))
    }
}
