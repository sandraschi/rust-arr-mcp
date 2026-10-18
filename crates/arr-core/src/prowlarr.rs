//! L1 Prowlarr driver: indexers + applications + search + history over Prowlarr
//! API v1 (`/api/v1`, `X-Api-Key`, 30 s timeout, 32 MiB cap). Same guard pattern
//! as `sonarr.rs`. `ProwlarrClient::demo()` is offline with fixtures.
//! Sync/history shapes reconstructed from the service client — pending live verify.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;
use crate::sonarr::{MAX_BODY_BYTES, REQUEST_TIMEOUT_SECS};

/// Prowlarr API path (legacy `PROWLARR_API_PATH`).
pub const API_PATH: &str = "/api/v1";

/// Prowlarr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct ProwlarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl ProwlarrClient {
    /// Live client. Empty URL or key is rejected.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "prowlarr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "prowlarr api_key is empty".to_string(),
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
                "prowlarr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!(
                "prowlarr {context}: unreachable ({})",
                self.base_url
            ));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!(
                    "prowlarr {context}: rejected credentials (401/403)"
                ));
            }
            return ArrError::Protocol(format!("prowlarr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("prowlarr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "prowlarr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
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
                    "prowlarr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "prowlarr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("prowlarr GET {path}: invalid JSON: {e}")))
    }

    async fn post_json(&self, path: &str, body: &Value) -> Result<Value, ArrError> {
        if self.demo {
            return Err(ArrError::Unsupported(
                "demo client: use fixture methods, not live HTTP".to_string(),
            ));
        }
        let resp = self
            .http
            .post(self.url(path))
            .header("X-Api-Key", &self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| self.classify("POST", e))?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| self.classify("POST body", e))?;
        self.check_body("POST", &bytes).await?;
        if !status.is_success() {
            return Err(ArrError::Protocol(format!(
                "prowlarr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("prowlarr POST {path}: invalid JSON: {e}")))
    }

    async fn put_json(&self, path: &str, body: &Value) -> Result<Value, ArrError> {
        if self.demo {
            return Err(ArrError::Unsupported(
                "demo client: use fixture methods, not live HTTP".to_string(),
            ));
        }
        let resp = self
            .http
            .put(self.url(path))
            .header("X-Api-Key", &self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| self.classify("PUT", e))?;
        let status = resp.status();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| self.classify("PUT body", e))?;
        self.check_body("PUT", &bytes).await?;
        if !status.is_success() {
            return Err(ArrError::Protocol(format!(
                "prowlarr PUT {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("prowlarr PUT {path}: invalid JSON: {e}")))
    }

    async fn delete(&self, path: &str) -> Result<(), ArrError> {
        if self.demo {
            return Err(ArrError::Unsupported(
                "demo client: use fixture methods, not live HTTP".to_string(),
            ));
        }
        let resp = self
            .http
            .delete(self.url(path))
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .map_err(|e| self.classify("DELETE", e))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ArrError::Protocol(format!(
                "prowlarr DELETE {path}: HTTP {status}"
            )));
        }
        Ok(())
    }

    // ---- indexers (8 ops) ----

    /// List indexers (`GET /indexer`). Demo returns two fixtures.
    pub async fn indexers_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(demo_indexers());
        }
        self.get_json("/indexer", &[]).await
    }

    /// Get one indexer (`GET /indexer/{id}`).
    pub async fn indexer_get(&self, indexer_id: i64) -> Result<Value, ArrError> {
        if indexer_id <= 0 {
            return Err(ArrError::InvalidInput(
                "indexer_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_indexer_get(indexer_id);
        }
        self.get_json(&format!("/indexer/{indexer_id}"), &[]).await
    }

    /// Add an indexer (`POST /indexer` with a field map). Demo echoes.
    pub async fn indexer_add(&self, fields: Value) -> Result<Value, ArrError> {
        if !fields.is_object() {
            return Err(ArrError::InvalidInput(
                "fields must be an object for add".to_string(),
            ));
        }
        if self.demo {
            let mut rec = Map::new();
            rec.insert("id".to_string(), Value::from(9006));
            if let Some(obj) = fields.as_object() {
                for (k, v) in obj {
                    rec.insert(k.clone(), v.clone());
                }
            }
            return Ok(Value::Object(rec));
        }
        self.post_json("/indexer", &fields).await
    }

    /// Update an indexer (GET then PUT, legacy pattern).
    pub async fn indexer_update(&self, indexer_id: i64, fields: Value) -> Result<Value, ArrError> {
        if indexer_id <= 0 {
            return Err(ArrError::InvalidInput(
                "indexer_id is required for update".to_string(),
            ));
        }
        if !fields.is_object() {
            return Err(ArrError::InvalidInput(
                "fields must be an object for update".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_indexer_get(indexer_id)?;
            if let (Some(obj), Some(patch)) = (rec.as_object_mut(), fields.as_object()) {
                for (k, v) in patch {
                    obj.insert(k.clone(), v.clone());
                }
            }
            return Ok(rec);
        }
        let mut current = self.indexer_get(indexer_id).await?;
        if let (Some(obj), Some(patch)) = (current.as_object_mut(), fields.as_object()) {
            for (k, v) in patch {
                obj.insert(k.clone(), v.clone());
            }
        }
        self.put_json(&format!("/indexer/{indexer_id}"), &current)
            .await
    }

    /// Delete an indexer (`DELETE /indexer/{id}`). Demo acknowledges.
    pub async fn indexer_delete(&self, indexer_id: i64) -> Result<Value, ArrError> {
        if indexer_id <= 0 {
            return Err(ArrError::InvalidInput(
                "indexer_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": indexer_id}));
        }
        self.delete(&format!("/indexer/{indexer_id}")).await?;
        Ok(serde_json::json!({"deleted": indexer_id}))
    }

    /// Test one indexer (`POST /indexer/test`). Demo acknowledges.
    pub async fn indexer_test(&self, indexer_id: i64) -> Result<Value, ArrError> {
        if indexer_id <= 0 {
            return Err(ArrError::InvalidInput(
                "indexer_id is required for test".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"tested": indexer_id, "ok": true}));
        }
        let indexer = self.indexer_get(indexer_id).await?;
        self.post_json("/indexer/test", &indexer).await
    }

    /// Test all indexers (`POST /indexer/testall`). Demo acknowledges.
    pub async fn indexers_test_all(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!({"tested": "all", "ok": true}));
        }
        self.post_json("/indexer/testall", &serde_json::json!({}))
            .await
    }

    /// Indexer schema (`GET /indexer/schema`). Demo returns one row.
    pub async fn indexer_schema(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([{"implementation": "Demo", "fields": []}]));
        }
        self.get_json("/indexer/schema", &[]).await
    }

    // ---- unified search ----

    /// Unified search across all indexers (`GET /search?query=`). Demo returns one hit.
    pub async fn search(&self, query: &str) -> Result<Value, ArrError> {
        if query.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "query is required for search".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!([
                {"title": "Demo.Release.2026", "indexer": "demo-indexer"}
            ]));
        }
        self.get_json("/search", &[("query".to_string(), query.to_string())])
            .await
    }

    // ---- applications (5 ops) ----

    /// List synced applications (`GET /applications`). Demo returns one row.
    pub async fn applications_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([{"id": 51, "name": "Demo Sonarr", "synced": true}]));
        }
        self.get_json("/applications", &[]).await
    }

    /// Get one application (`GET /applications/{id}`).
    pub async fn application_get(&self, app_id: i64) -> Result<Value, ArrError> {
        if app_id <= 0 {
            return Err(ArrError::InvalidInput(
                "app_id is required for get".to_string(),
            ));
        }
        if self.demo {
            if app_id == 51 {
                return Ok(serde_json::json!({"id": 51, "name": "Demo Sonarr", "synced": true}));
            }
            return Err(ArrError::Protocol(format!(
                "demo: application {app_id} not found"
            )));
        }
        self.get_json(&format!("/applications/{app_id}"), &[]).await
    }

    /// Sync one application. Path reconstructed (`POST /applications/{id}/sync`) — pending live verify.
    pub async fn application_sync(&self, app_id: i64) -> Result<Value, ArrError> {
        if app_id <= 0 {
            return Err(ArrError::InvalidInput(
                "app_id is required for sync".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"synced": app_id}));
        }
        self.post_json(
            &format!("/applications/{app_id}/sync"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Sync all applications. Path reconstructed (`POST /applications/sync`) — pending live verify.
    pub async fn applications_sync_all(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!({"synced": "all"}));
        }
        self.post_json("/applications/sync", &serde_json::json!({}))
            .await
    }

    /// Test one application. Shape reconstructed (`POST /applications/{id}/test`) — pending live verify.
    pub async fn application_test(&self, app_id: i64) -> Result<Value, ArrError> {
        if app_id <= 0 {
            return Err(ArrError::InvalidInput(
                "app_id is required for test".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"tested": app_id, "ok": true}));
        }
        self.post_json(
            &format!("/applications/{app_id}/test"),
            &serde_json::json!({}),
        )
        .await
    }

    // ---- history (3 ops) ----

    /// History list (`GET /history`). Demo returns one row.
    pub async fn history_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([{"id": 71, "source": "demo-indexer"}]));
        }
        self.get_json("/history", &[]).await
    }

    /// History since a date (`GET /history?since=`). Shape reconstructed — pending live verify.
    pub async fn history_since(&self, since: &str) -> Result<Value, ArrError> {
        if since.trim().is_empty() {
            return Err(ArrError::InvalidInput("since date is required".to_string()));
        }
        if self.demo {
            return Ok(serde_json::json!([{"id": 71, "source": "demo-indexer"}]));
        }
        self.get_json("/history", &[("since".to_string(), since.to_string())])
            .await
    }

    /// History by indexer (`GET /history?indexerId=`). Shape reconstructed — pending live verify.
    pub async fn history_by_indexer(&self, indexer_id: i64) -> Result<Value, ArrError> {
        if indexer_id <= 0 {
            return Err(ArrError::InvalidInput("indexer_id is required".to_string()));
        }
        if self.demo {
            return Ok(serde_json::json!([{"id": 71, "source": "demo-indexer"}]));
        }
        self.get_json(
            "/history",
            &[("indexerId".to_string(), indexer_id.to_string())],
        )
        .await
    }
}

// ---- demo fixtures (offline, deterministic) ----

fn demo_indexers() -> Value {
    serde_json::json!([
        {"id": 41, "name": "Demo Indexer", "enable": true},
        {"id": 42, "name": "Second Indexer", "enable": false},
    ])
}

fn demo_indexer_get(indexer_id: i64) -> Result<Value, ArrError> {
    let all = demo_indexers();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(indexer_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: indexer {indexer_id} not found"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(ProwlarrClient::new("", "key").is_err());
        assert!(ProwlarrClient::new("http://x:9696", "").is_err());
    }

    #[tokio::test]
    async fn demo_indexers_crud() {
        let client = ProwlarrClient::demo();
        let list = client.indexers_list().await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        assert!(client.indexer_get(0).await.is_err());
        let added = client
            .indexer_add(serde_json::json!({"name": "New"}))
            .await
            .unwrap_or(Value::Null);
        assert_eq!(added.get("id").and_then(|v| v.as_i64()), Some(9006));
        assert!(client.indexer_add(Value::Null).await.is_err());
        let tested = client.indexer_test(41).await.unwrap_or(Value::Null);
        assert_eq!(tested.get("ok"), Some(&Value::Bool(true)));
        let all = client.indexers_test_all().await.unwrap_or(Value::Null);
        assert_eq!(all.get("ok"), Some(&Value::Bool(true)));
        let deleted = client.indexer_delete(41).await.unwrap_or(Value::Null);
        assert_eq!(deleted.get("deleted").and_then(|v| v.as_i64()), Some(41));
    }

    #[tokio::test]
    async fn demo_search_apps_history() {
        let client = ProwlarrClient::demo();
        assert!(client.search("").await.is_err());
        let hits = client.search("Dune").await.unwrap_or(Value::Null);
        assert_eq!(hits.as_array().map(Vec::len), Some(1));
        let apps = client.applications_list().await.unwrap_or(Value::Null);
        assert_eq!(apps.as_array().map(Vec::len), Some(1));
        let synced = client.application_sync(51).await.unwrap_or(Value::Null);
        assert_eq!(synced.get("synced").and_then(|v| v.as_i64()), Some(51));
        assert!(client.application_sync(0).await.is_err());
        let hist = client.history_list().await.unwrap_or(Value::Null);
        assert_eq!(hist.as_array().map(Vec::len), Some(1));
        assert!(client.history_since("").await.is_err());
        assert!(client.history_by_indexer(0).await.is_err());
    }
}
