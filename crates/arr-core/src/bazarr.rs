//! L1 Bazarr driver: subtitles + wanted + providers over Bazarr API (`/api`,
//! `X-Api-Key`, 30 s timeout, 32 MiB cap). Same guard pattern as `sonarr.rs`.
//! `BazarrClient::demo()` is offline with deterministic fixtures.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;
use crate::sonarr::{MAX_BODY_BYTES, REQUEST_TIMEOUT_SECS};

/// Bazarr API path (legacy `BAZARR_API_PATH`).
pub const API_PATH: &str = "/api";

/// Bazarr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct BazarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl BazarrClient {
    /// Live client. Empty URL or key is rejected.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "bazarr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "bazarr api_key is empty".to_string(),
            ));
        }
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| ArrError::Protocol(format!("http client build: {e}")))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            http,
            demo: false,
        })
    }

    /// Deterministic offline fixture client for CI/tests.
    pub fn demo() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            base_url: "http://demo.invalid".to_string(),
            api_key: String::new(),
            http,
            demo: true,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{API_PATH}{path}", self.base_url)
    }

    fn classify(&self, context: &str, err: reqwest::Error) -> ArrError {
        if err.is_timeout() {
            return ArrError::Timeout(format!(
                "bazarr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!("bazarr {context}: unreachable ({})", self.base_url));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!("bazarr {context}: rejected credentials (401/403)"));
            }
            return ArrError::Protocol(format!("bazarr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("bazarr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "bazarr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
                bytes.len()
            )));
        }
        Ok(())
    }

    async fn get_json(&self, path: &str, query: &[(String, String)]) -> Result<Value, ArrError> {
        if self.demo {
            return Err(ArrError::Unsupported(
                "demo client: use fixture methods, not live HTTP".to_string(),
            ));
        }
        let mut req = self
            .http
            .get(self.url(path))
            .header("X-Api-Key", &self.api_key);
        for (k, v) in query {
            req = req.query(&[(k, v)]);
        }
        let resp = req.send().await.map_err(|e| self.classify("GET", e))?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| self.classify("GET body", e))?;
        self.check_body("GET", &bytes).await?;
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return Err(ArrError::Auth(
                    "bazarr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "bazarr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("bazarr GET {path}: invalid JSON: {e}")))
    }

    async fn post_json(&self, path: &str, query: &[(String, String)]) -> Result<Value, ArrError> {
        if self.demo {
            return Err(ArrError::Unsupported(
                "demo client: use fixture methods, not live HTTP".to_string(),
            ));
        }
        let mut req = self
            .http
            .post(self.url(path))
            .header("X-Api-Key", &self.api_key);
        for (k, v) in query {
            req = req.query(&[(k, v)]);
        }
        let resp = req.send().await.map_err(|e| self.classify("POST", e))?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| self.classify("POST body", e))?;
        self.check_body("POST", &bytes).await?;
        if !status.is_success() {
            return Err(ArrError::Protocol(format!(
                "bazarr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("bazarr POST {path}: invalid JSON: {e}")))
    }

    /// Wanted subtitles: movies, episodes, or all (`GET /movies/wanted`,
    /// `/episodes/wanted`). Demo returns fixtures.
    pub async fn wanted(&self, kind: &str) -> Result<Value, ArrError> {
        match kind {
            "movies" | "episodes" | "all" => {}
            _ => {
                return Err(ArrError::InvalidInput(
                    "kind must be movies, episodes, or all".to_string(),
                ))
            }
        }
        if self.demo {
            return Ok(demo_wanted(kind));
        }
        if kind == "movies" {
            return self.get_json("/movies/wanted", &[]).await;
        }
        if kind == "episodes" {
            return self.get_json("/episodes/wanted", &[]).await;
        }
        let movies = self
            .get_json("/movies/wanted", &[])
            .await
            .unwrap_or(Value::Null);
        let episodes = self
            .get_json("/episodes/wanted", &[])
            .await
            .unwrap_or(Value::Null);
        Ok(serde_json::json!({"movies": movies, "episodes": episodes}))
    }

    /// Search subtitles (`GET /subtitles` with media identifiers).
    /// Identifier shapes reconstructed from the service client — pending live verify.
    pub async fn search(
        &self,
        series_id: Option<i64>,
        episode_id: Option<i64>,
        movie_id: Option<i64>,
    ) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([
                {"id": 301, "name": "Demo.Sub.srt", "language": "en"}
            ]));
        }
        let mut query = Vec::new();
        if let Some(id) = series_id {
            query.push(("seriesid".to_string(), id.to_string()));
        }
        if let Some(id) = episode_id {
            query.push(("episodeid".to_string(), id.to_string()));
        }
        if let Some(id) = movie_id {
            query.push(("movieid".to_string(), id.to_string()));
        }
        self.get_json("/subtitles", &query).await
    }

    /// Download one subtitle (`POST /subtitles` with the subtitle id).
    pub async fn download(&self, subtitle_id: &str) -> Result<Value, ArrError> {
        if subtitle_id.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "subtitle_id is required for download".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"downloaded": subtitle_id}));
        }
        self.post_json("/subtitles", &[("id".to_string(), subtitle_id.to_string())])
            .await
    }

    /// Download history (`GET /history`). Demo returns one row.
    pub async fn history(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([
                {"id": 302, "action": "download", "language": "en"}
            ]));
        }
        self.get_json("/history", &[]).await
    }

    /// Configured providers (`GET /providers`). Demo returns two rows.
    pub async fn providers(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([
                {"name": "demo-provider-a", "enabled": true},
                {"name": "demo-provider-b", "enabled": false},
            ]));
        }
        self.get_json("/providers", &[]).await
    }

    /// Supported languages (`GET /languages`). Demo returns two rows.
    pub async fn languages(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([
                {"code": "en", "name": "English"},
                {"code": "de", "name": "German"},
            ]));
        }
        self.get_json("/languages", &[]).await
    }
}

fn demo_wanted(kind: &str) -> Value {
    match kind {
        "movies" => serde_json::json!([{"id": 7, "title": "Demo Movie"}]),
        "episodes" => serde_json::json!([{"id": 456, "title": "Pilot"}]),
        _ => serde_json::json!({
            "movies": [{"id": 7, "title": "Demo Movie"}],
            "episodes": [{"id": 456, "title": "Pilot"}],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(BazarrClient::new("", "key").is_err());
        assert!(BazarrClient::new("http://x:6767", "").is_err());
    }

    #[tokio::test]
    async fn demo_wanted_guards_kind() {
        let client = BazarrClient::demo();
        assert!(client.wanted("shows").await.is_err());
        let movies = client.wanted("movies").await.unwrap_or(Value::Null);
        assert_eq!(movies.as_array().map(Vec::len), Some(1));
    }

    #[tokio::test]
    async fn demo_search_download_history() {
        let client = BazarrClient::demo();
        let hits = client
            .search(None, Some(456), None)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(hits.as_array().map(Vec::len), Some(1));
        assert!(client.download("").await.is_err());
        let done = client.download("301").await.unwrap_or(Value::Null);
        assert_eq!(done.get("downloaded").and_then(|v| v.as_str()), Some("301"));
        let providers = client.providers().await.unwrap_or(Value::Null);
        assert_eq!(providers.as_array().map(Vec::len), Some(2));
        let languages = client.languages().await.unwrap_or(Value::Null);
        assert_eq!(languages.as_array().map(Vec::len), Some(2));
    }
}
