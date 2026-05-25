use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::Value;

/// Generic API client for the marketing backend.
#[derive(Clone)]
pub struct ApiClient {
    pub http: Client,
    pub base_url: String,
    pub auth_header: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String, auth_header: Option<String>) -> Self {
        Self { http: Client::new(), base_url: base_url.trim_end_matches('/').to_string(), auth_header }
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let mut req = self.http.get(format!("{}{}", self.base_url, path));
        if let Some(ref auth) = self.auth_header { req = req.header("Authorization", auth); }
        let resp = req.send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let mut req = self.http.post(format!("{}{}", self.base_url, path)).json(body);
        if let Some(ref auth) = self.auth_header { req = req.header("Authorization", auth); }
        let resp = req.send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        let mut req = self.http.patch(format!("{}{}", self.base_url, path)).json(body);
        if let Some(ref auth) = self.auth_header { req = req.header("Authorization", auth); }
        let resp = req.send().await?;
        if !resp.status().is_success() { bail!("API {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }
}

/// Detect backend from env vars.
pub fn detect_backend() -> Result<ApiClient> {
    // HubSpot
    if let Ok(token) = std::env::var("HUBSPOT_ACCESS_TOKEN") {
        tracing::info!("Using HubSpot backend");
        return Ok(ApiClient::new("https://api.hubapi.com".into(), Some(format!("Bearer {}", token))));
    }
    // Mailchimp
    if let Ok(key) = std::env::var("MAILCHIMP_API_KEY") {
        let dc = key.split('-').last().unwrap_or("us1");
        tracing::info!("Using Mailchimp backend (dc: {})", dc);
        return Ok(ApiClient::new(format!("https://{}.api.mailchimp.com/3.0", dc), Some(format!("Bearer {}", key))));
    }
    // Custom API
    if let Ok(url) = std::env::var("MARKETING_API_URL") {
        let auth = std::env::var("MARKETING_API_KEY").ok().map(|k| format!("Bearer {}", k));
        tracing::info!("Using custom marketing API backend");
        return Ok(ApiClient::new(url, auth));
    }
    bail!("No marketing backend configured. Set HUBSPOT_ACCESS_TOKEN, MAILCHIMP_API_KEY, or MARKETING_API_URL")
}
