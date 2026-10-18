//! L1 Overseerr driver: requests + search + users over Overseerr API v1
//! (`/api/v1` absolute paths, `X-Api-Key`, 30 s timeout, 32 MiB cap). Same guard
//! pattern as `sonarr.rs`. `OverseerrClient::demo()` is offline with fixtures.
//! Create payload (`{mediaType, mediaId, seasons}`) reconstructed from the
//! service client — pending live verify.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;
use crate::sonarr::{MAX_BODY_BYTES, REQUEST_TIMEOUT_SECS};

/// Overseerr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct OverseerrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl OverseerrClient {
    /// Live client. Empty URL or key is rejected.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "overseerr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "overseerr api_key is empty".to_string(),
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
        // Overseerr paths are absolute (`/api/v1/...`), no per-service prefix.
        format!("{}{path}", self.base_url)
    }

    fn classify(&self, context: &str, err: reqwest::Error) -> ArrError {
        if err.is_timeout() {
            return ArrError::Timeout(format!(
                "overseerr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!(
                "overseerr {context}: unreachable ({})",
                self.base_url
            ));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!(
                    "overseerr {context}: rejected credentials (401/403)"
                ));
            }
            return ArrError::Protocol(format!("overseerr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("overseerr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "overseerr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
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
                    "overseerr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "overseerr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("overseerr GET {path}: invalid JSON: {e}")))
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
                "overseerr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("overseerr POST {path}: invalid JSON: {e}")))
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
                "overseerr DELETE {path}: HTTP {status}"
            )));
        }
        Ok(())
    }

    // ---- requests (8 ops) ----

    /// List requests (`GET /api/v1/request`). Demo returns two fixtures.
    pub async fn requests_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(demo_requests());
        }
        self.get_json("/api/v1/request", &[]).await
    }

    /// Get one request (`GET /api/v1/request/{id}`).
    pub async fn request_get(&self, request_id: i64) -> Result<Value, ArrError> {
        if request_id <= 0 {
            return Err(ArrError::InvalidInput(
                "request_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_request_get(request_id);
        }
        self.get_json(&format!("/api/v1/request/{request_id}"), &[])
            .await
    }

    /// Create a request (`POST /api/v1/request`). Demo echoes.
    pub async fn request_create(
        &self,
        media_type: &str,
        media_id: i64,
        seasons: &[i64],
    ) -> Result<Value, ArrError> {
        match media_type {
            "movie" | "tv" => {}
            _ => {
                return Err(ArrError::InvalidInput(
                    "media_type must be movie or tv".to_string(),
                ))
            }
        }
        if media_id <= 0 {
            return Err(ArrError::InvalidInput(
                "media_id is required for create".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({
                "id": 9007,
                "media": {"mediaType": media_type, "tmdbId": media_id},
                "seasons": seasons,
                "status": "pending",
            }));
        }
        let body = serde_json::json!({
            "mediaType": media_type,
            "mediaId": media_id,
            "seasons": seasons,
        });
        self.post_json("/api/v1/request", &body).await
    }

    /// Approve a request (`POST /api/v1/request/{id}/approve`). Demo acknowledges.
    pub async fn request_approve(&self, request_id: i64) -> Result<Value, ArrError> {
        if request_id <= 0 {
            return Err(ArrError::InvalidInput(
                "request_id is required for approve".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"id": request_id, "status": "approved"}));
        }
        self.post_json(
            &format!("/api/v1/request/{request_id}/approve"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Decline a request (`POST /api/v1/request/{id}/decline`). Demo acknowledges.
    pub async fn request_decline(&self, request_id: i64) -> Result<Value, ArrError> {
        if request_id <= 0 {
            return Err(ArrError::InvalidInput(
                "request_id is required for decline".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"id": request_id, "status": "declined"}));
        }
        self.post_json(
            &format!("/api/v1/request/{request_id}/decline"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Delete a request (`DELETE /api/v1/request/{id}`). Demo acknowledges.
    pub async fn request_delete(&self, request_id: i64) -> Result<Value, ArrError> {
        if request_id <= 0 {
            return Err(ArrError::InvalidInput(
                "request_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": request_id}));
        }
        self.delete(&format!("/api/v1/request/{request_id}"))
            .await?;
        Ok(serde_json::json!({"deleted": request_id}))
    }

    /// Request count (`GET /api/v1/request/count`). Demo returns two.
    pub async fn requests_count(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!({"total": 2, "pending": 1}));
        }
        self.get_json("/api/v1/request/count", &[]).await
    }

    /// Pending requests (client-side filter over list; mirrors legacy `pending` op).
    pub async fn requests_pending(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([
                {"id": 81, "status": "pending", "media": {"title": "Demo Request"}}
            ]));
        }
        let all = self.requests_list().await?;
        let items = all
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.get("status").and_then(|s| s.as_str()) == Some("pending"));
        Ok(Value::Array(items.collect()))
    }

    // ---- search ----

    /// Media search (`GET /api/v1/search?query=&page=`). Demo returns one hit.
    pub async fn search(&self, query: &str) -> Result<Value, ArrError> {
        if query.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "query is required for search".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!([
                {"mediaType": "movie", "title": "Demo Hit"}
            ]));
        }
        self.get_json(
            "/api/v1/search",
            &[
                ("query".to_string(), query.to_string()),
                ("page".to_string(), "1".to_string()),
            ],
        )
        .await
    }

    // ---- users (3 ops) ----

    /// List users (`GET /api/v1/user`). Demo returns one row.
    pub async fn users_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([{"id": 1, "displayName": "Demo Admin"}]));
        }
        self.get_json("/api/v1/user", &[]).await
    }

    /// Get one user (`GET /api/v1/user/{id}`).
    pub async fn user_get(&self, user_id: i64) -> Result<Value, ArrError> {
        if user_id <= 0 {
            return Err(ArrError::InvalidInput(
                "user_id is required for get".to_string(),
            ));
        }
        if self.demo {
            if user_id == 1 {
                return Ok(serde_json::json!({"id": 1, "displayName": "Demo Admin"}));
            }
            return Err(ArrError::Protocol(format!(
                "demo: user {user_id} not found"
            )));
        }
        self.get_json(&format!("/api/v1/user/{user_id}"), &[]).await
    }

    /// Requests by user (`GET /api/v1/user/{id}/requests`). Demo returns one row.
    pub async fn user_requests(&self, user_id: i64) -> Result<Value, ArrError> {
        if user_id <= 0 {
            return Err(ArrError::InvalidInput("user_id is required".to_string()));
        }
        if self.demo {
            return Ok(serde_json::json!([
                {"id": 81, "status": "pending", "requestedBy": user_id}
            ]));
        }
        self.get_json(&format!("/api/v1/user/{user_id}/requests"), &[])
            .await
    }
}

// ---- demo fixtures (offline, deterministic) ----

fn demo_requests() -> Value {
    serde_json::json!([
        {"id": 81, "status": "pending", "media": {"title": "Demo Request"}},
        {"id": 82, "status": "approved", "media": {"title": "Old Request"}},
    ])
}

fn demo_request_get(request_id: i64) -> Result<Value, ArrError> {
    let all = demo_requests();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(request_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: request {request_id} not found"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(OverseerrClient::new("", "key").is_err());
        assert!(OverseerrClient::new("http://x:5055", "").is_err());
    }

    #[tokio::test]
    async fn demo_requests_lifecycle() {
        let client = OverseerrClient::demo();
        let list = client.requests_list().await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        assert!(client.request_get(0).await.is_err());
        let created = client
            .request_create("movie", 438631, &[])
            .await
            .unwrap_or(Value::Null);
        assert_eq!(created.get("id").and_then(|v| v.as_i64()), Some(9007));
        assert!(client.request_create("game", 1, &[]).await.is_err());
        let approved = client.request_approve(81).await.unwrap_or(Value::Null);
        assert_eq!(
            approved.get("status").and_then(|v| v.as_str()),
            Some("approved")
        );
        let declined = client.request_decline(81).await.unwrap_or(Value::Null);
        assert_eq!(
            declined.get("status").and_then(|v| v.as_str()),
            Some("declined")
        );
        let count = client.requests_count().await.unwrap_or(Value::Null);
        assert_eq!(count.get("total").and_then(|v| v.as_i64()), Some(2));
        let pending = client.requests_pending().await.unwrap_or(Value::Null);
        assert_eq!(pending.as_array().map(Vec::len), Some(1));
    }

    #[tokio::test]
    async fn demo_search_and_users() {
        let client = OverseerrClient::demo();
        assert!(client.search("").await.is_err());
        let hits = client.search("Dune").await.unwrap_or(Value::Null);
        assert_eq!(hits.as_array().map(Vec::len), Some(1));
        let users = client.users_list().await.unwrap_or(Value::Null);
        assert_eq!(users.as_array().map(Vec::len), Some(1));
        assert!(client.user_get(0).await.is_err());
        let mine = client.user_requests(1).await.unwrap_or(Value::Null);
        assert_eq!(mine.as_array().map(Vec::len), Some(1));
    }
}
