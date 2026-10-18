//! L1 Radarr driver: movie lifecycle over Radarr API v3 (`/api/v3`,
//! `X-Api-Key`, 30 s timeout, 32 MiB cap). Same guard pattern as `sonarr.rs`:
//! required fields, id-mismatch blocks search. `RadarrClient::demo()` is offline.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;
use crate::sonarr::{MAX_BODY_BYTES, REQUEST_TIMEOUT_SECS};

/// Radarr API path (legacy `RADARR_API_PATH`).
pub const API_PATH: &str = "/api/v3";

/// Radarr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct RadarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl RadarrClient {
    /// Live client. Empty URL or key is rejected.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "radarr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "radarr api_key is empty".to_string(),
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
                "radarr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!("radarr {context}: unreachable ({})", self.base_url));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!("radarr {context}: rejected credentials (401/403)"));
            }
            return ArrError::Protocol(format!("radarr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("radarr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "radarr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
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
                    "radarr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "radarr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("radarr GET {path}: invalid JSON: {e}")))
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
                "radarr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("radarr POST {path}: invalid JSON: {e}")))
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
                "radarr PUT {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("radarr PUT {path}: invalid JSON: {e}")))
    }

    async fn delete(&self, path: &str, query: &[(String, String)]) -> Result<(), ArrError> {
        if self.demo {
            return Err(ArrError::Unsupported(
                "demo client: use fixture methods, not live HTTP".to_string(),
            ));
        }
        let mut req = self
            .http
            .delete(self.url(path))
            .header("X-Api-Key", &self.api_key);
        for (k, v) in query {
            req = req.query(&[(k, v)]);
        }
        let resp = req.send().await.map_err(|e| self.classify("DELETE", e))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ArrError::Protocol(format!(
                "radarr DELETE {path}: HTTP {status}"
            )));
        }
        Ok(())
    }

    /// List all movies (`GET /movie`). Demo returns two fixtures.
    pub async fn movies_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(demo_movies_list());
        }
        self.get_json("/movie", &[]).await
    }

    /// Lookup metadata by term (`GET /movie/lookup?term=`). Demo filters fixtures.
    pub async fn movies_lookup(&self, term: &str) -> Result<Value, ArrError> {
        if term.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "term is required for lookup".to_string(),
            ));
        }
        if self.demo {
            let needle = term.to_ascii_lowercase();
            let all = demo_movies_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|m| {
                    m.get("title")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_ascii_lowercase().contains(&needle))
                        .unwrap_or(false)
                });
            return Ok(Value::Array(items.collect()));
        }
        self.get_json("/movie/lookup", &[("term".to_string(), term.to_string())])
            .await
    }

    /// Get one movie (`GET /movie/{id}`).
    pub async fn movie_get(&self, movie_id: i64) -> Result<Value, ArrError> {
        if movie_id <= 0 {
            return Err(ArrError::InvalidInput(
                "movie_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_movie_get(movie_id);
        }
        self.get_json(&format!("/movie/{movie_id}"), &[]).await
    }

    /// Add a movie (`POST /movie`). Demo echoes a constructed record.
    pub async fn movie_add(
        &self,
        tmdb_id: i64,
        title: &str,
        quality_profile_id: i64,
        root_folder_path: &str,
        monitored: bool,
        search_for_movie: bool,
    ) -> Result<Value, ArrError> {
        if tmdb_id <= 0 || quality_profile_id <= 0 || root_folder_path.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "tmdb_id, quality_profile_id, and root_folder_path are required".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({
                "id": 9002,
                "tmdbId": tmdb_id,
                "title": title,
                "qualityProfileId": quality_profile_id,
                "rootFolderPath": root_folder_path,
                "monitored": monitored,
                "addOptions": {"searchForMovie": search_for_movie},
            }));
        }
        let body = serde_json::json!({
            "tmdbId": tmdb_id,
            "title": title,
            "qualityProfileId": quality_profile_id,
            "rootFolderPath": root_folder_path,
            "monitored": monitored,
            "addOptions": {"searchForMovie": search_for_movie},
        });
        self.post_json("/movie", &body).await
    }

    /// Trigger `MoviesSearch` after verifying the ID round-trips (legacy guard).
    pub async fn movie_search(&self, movie_id: i64) -> Result<Value, ArrError> {
        if movie_id <= 0 {
            return Err(ArrError::InvalidInput(
                "a positive movie_id is required for search".to_string(),
            ));
        }
        let movie = self.movie_get(movie_id).await?;
        let returned = movie.get("id").and_then(|v| v.as_i64()).unwrap_or(-1);
        if returned != movie_id {
            return Err(ArrError::Protocol(format!(
                "mismatched movie ID for requested movie {movie_id}; search was not started"
            )));
        }
        if self.demo {
            return Ok(serde_json::json!({"name": "MoviesSearch", "movieIds": [movie_id]}));
        }
        self.post_json(
            "/command",
            &serde_json::json!({"name": "MoviesSearch", "movieIds": [movie_id]}),
        )
        .await
    }

    /// Delete a movie (`DELETE /movie/{id}`). Demo acknowledges.
    pub async fn movie_delete(
        &self,
        movie_id: i64,
        delete_files: bool,
        add_exclusion: bool,
    ) -> Result<Value, ArrError> {
        if movie_id <= 0 {
            return Err(ArrError::InvalidInput(
                "movie_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": movie_id}));
        }
        self.delete(
            &format!("/movie/{movie_id}"),
            &[
                ("deleteFiles".to_string(), delete_files.to_string()),
                (
                    "addImportListExclusion".to_string(),
                    add_exclusion.to_string(),
                ),
            ],
        )
        .await?;
        Ok(serde_json::json!({"deleted": movie_id}))
    }

    /// Update monitored flag (GET then PUT, legacy pattern).
    pub async fn movie_update(&self, movie_id: i64, monitored: bool) -> Result<Value, ArrError> {
        if movie_id <= 0 {
            return Err(ArrError::InvalidInput(
                "movie_id is required for update".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_movie_get(movie_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut movie = self.movie_get(movie_id).await?;
        if let Some(obj) = movie.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/movie/{movie_id}"), &movie).await
    }

    /// Manual import candidates (`GET /manualimport?folder=`). Demo returns one row.
    pub async fn movies_import(&self, folder: Option<&str>) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!([
                {"path": folder.unwrap_or("/movies/incoming"), "movie": {"id": 7, "title": "Demo Import"}}
            ]));
        }
        let mut query = Vec::new();
        if let Some(folder) = folder {
            if !folder.trim().is_empty() {
                query.push(("folder".to_string(), folder.to_string()));
            }
        }
        self.get_json("/manualimport", &query).await
    }
}

// ---- demo fixtures (offline, deterministic) ----

fn demo_movies_list() -> Value {
    serde_json::json!([
        {"id": 7, "title": "Demo Movie", "tmdbId": 438631, "monitored": true},
        {"id": 8, "title": "Another Film", "tmdbId": 12345, "monitored": false},
    ])
}

fn demo_movie_get(movie_id: i64) -> Result<Value, ArrError> {
    let all = demo_movies_list();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(movie_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: movie {movie_id} not found"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(RadarrClient::new("", "key").is_err());
        assert!(RadarrClient::new("http://x:7878", "").is_err());
    }

    #[tokio::test]
    async fn demo_list_and_lookup() {
        let client = RadarrClient::demo();
        let list = client.movies_list().await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        let hits = client.movies_lookup("demo").await.unwrap_or(Value::Null);
        assert_eq!(hits.as_array().map(Vec::len), Some(1));
        assert!(client.movies_lookup("").await.is_err());
    }

    #[tokio::test]
    async fn demo_search_verifies_id() {
        let client = RadarrClient::demo();
        let cmd = client.movie_search(7).await.unwrap_or(Value::Null);
        assert_eq!(
            cmd.get("name").and_then(|v| v.as_str()),
            Some("MoviesSearch")
        );
        assert!(client.movie_search(0).await.is_err());
        assert!(client.movie_search(999).await.is_err());
    }

    #[tokio::test]
    async fn demo_add_update_delete_import() {
        let client = RadarrClient::demo();
        let added = client
            .movie_add(438631, "Dune", 1, "/movies", true, true)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(added.get("tmdbId").and_then(|v| v.as_i64()), Some(438631));
        assert!(client
            .movie_add(0, "x", 1, "/movies", true, true)
            .await
            .is_err());
        let updated = client.movie_update(7, false).await.unwrap_or(Value::Null);
        assert_eq!(updated.get("monitored"), Some(&Value::Bool(false)));
        let deleted = client
            .movie_delete(7, false, false)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(deleted.get("deleted").and_then(|v| v.as_i64()), Some(7));
        let imp = client
            .movies_import(Some("/movies/incoming"))
            .await
            .unwrap_or(Value::Null);
        assert_eq!(imp.as_array().map(Vec::len), Some(1));
    }
}
