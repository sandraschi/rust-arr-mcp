//! L1 Sonarr driver: series + episode management over Sonarr API v3.
//! Wire essence from `arr-mcp/src/arr_mcp/{tools/sonarr_tools.py,services/sonarr_service.py}`:
//! base `{url}/api/v3`, auth `X-Api-Key` header, 30 s timeout, 32 MiB payload cap.
//! Validation mirrors legacy (required fields, id-mismatch blocks search).
//! `SonarrClient::demo()` serves deterministic fixtures with zero network.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;

/// Maximum JSON response body accepted (32 MiB). Larger bodies are rejected
/// before parsing so a runaway library can never OOM the process.
pub const MAX_BODY_BYTES: usize = 32 * 1024 * 1024;

/// Per-request timeout (mirrors legacy `DEFAULT_TIMEOUT = 30`).
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Sonarr API path (legacy `SONARR_API_PATH`).
pub const API_PATH: &str = "/api/v3";

/// Sonarr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct SonarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl SonarrClient {
    /// Live client. Empty URL or key is rejected (no silent misconfiguration).
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "sonarr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "sonarr api_key is empty".to_string(),
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
                "sonarr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!("sonarr {context}: unreachable ({})", self.base_url));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!("sonarr {context}: rejected credentials (401/403)"));
            }
            return ArrError::Protocol(format!("sonarr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("sonarr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "sonarr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
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
                    "sonarr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "sonarr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("sonarr GET {path}: invalid JSON: {e}")))
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
                "sonarr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("sonarr POST {path}: invalid JSON: {e}")))
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
                "sonarr PUT {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("sonarr PUT {path}: invalid JSON: {e}")))
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
                "sonarr DELETE {path}: HTTP {status}"
            )));
        }
        Ok(())
    }

    // ---- series ----

    /// List all series (`GET /series`). Demo returns two fixtures.
    pub async fn series_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(demo_series_list());
        }
        self.get_json("/series", &[]).await
    }

    /// Lookup metadata by term (`GET /series/lookup?term=`). Demo filters fixtures.
    pub async fn series_lookup(&self, term: &str) -> Result<Value, ArrError> {
        if term.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "term is required for lookup".to_string(),
            ));
        }
        if self.demo {
            let needle = term.to_ascii_lowercase();
            let all = demo_series_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|s| {
                    s.get("title")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_ascii_lowercase().contains(&needle))
                        .unwrap_or(false)
                });
            return Ok(Value::Array(items.collect()));
        }
        self.get_json("/series/lookup", &[("term".to_string(), term.to_string())])
            .await
    }

    /// Get one series by ID (`GET /series/{id}`).
    pub async fn series_get(&self, series_id: i64) -> Result<Value, ArrError> {
        if series_id <= 0 {
            return Err(ArrError::InvalidInput(
                "series_id must be positive".to_string(),
            ));
        }
        if self.demo {
            return demo_series_get(series_id);
        }
        self.get_json(&format!("/series/{series_id}"), &[]).await
    }

    /// Add a series (`POST /series`). Demo echoes a constructed record.
    pub async fn series_add(
        &self,
        tvdb_id: i64,
        title: &str,
        quality_profile_id: i64,
        root_folder_path: &str,
        monitored: bool,
        search_for_missing: bool,
    ) -> Result<Value, ArrError> {
        if tvdb_id <= 0 || quality_profile_id <= 0 || root_folder_path.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "tvdb_id, quality_profile_id, and root_folder_path are required".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({
                "id": 9001,
                "tvdbId": tvdb_id,
                "title": title,
                "qualityProfileId": quality_profile_id,
                "rootFolderPath": root_folder_path,
                "monitored": monitored,
                "addOptions": {"searchForMissingEpisodes": search_for_missing},
            }));
        }
        let body = serde_json::json!({
            "tvdbId": tvdb_id,
            "title": title,
            "qualityProfileId": quality_profile_id,
            "rootFolderPath": root_folder_path,
            "monitored": monitored,
            "addOptions": {"searchForMissingEpisodes": search_for_missing},
        });
        self.post_json("/series", &body).await
    }

    /// Trigger `SeriesSearch` after verifying the ID round-trips (legacy guard).
    pub async fn series_search(&self, series_id: i64) -> Result<Value, ArrError> {
        if series_id <= 0 {
            return Err(ArrError::InvalidInput(
                "a positive series_id is required for search".to_string(),
            ));
        }
        let series = self.series_get(series_id).await?;
        let returned = series.get("id").and_then(|v| v.as_i64()).unwrap_or(-1);
        if returned != series_id {
            return Err(ArrError::Protocol(format!(
                "mismatched series ID for requested series {series_id}; search was not started"
            )));
        }
        if self.demo {
            return Ok(serde_json::json!({"name": "SeriesSearch", "seriesId": series_id}));
        }
        self.post_json(
            "/command",
            &serde_json::json!({"name": "SeriesSearch", "seriesId": series_id}),
        )
        .await
    }

    /// Delete a series (`DELETE /series/{id}`). Demo acknowledges.
    pub async fn series_delete(
        &self,
        series_id: i64,
        delete_files: bool,
        add_exclusion: bool,
    ) -> Result<Value, ArrError> {
        if series_id <= 0 {
            return Err(ArrError::InvalidInput(
                "series_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": series_id}));
        }
        self.delete(
            &format!("/series/{series_id}"),
            &[
                ("deleteFiles".to_string(), delete_files.to_string()),
                (
                    "addImportListExclusion".to_string(),
                    add_exclusion.to_string(),
                ),
            ],
        )
        .await?;
        Ok(serde_json::json!({"deleted": series_id}))
    }

    /// Update monitored flag (GET then PUT, legacy pattern).
    pub async fn series_update(&self, series_id: i64, monitored: bool) -> Result<Value, ArrError> {
        if series_id <= 0 {
            return Err(ArrError::InvalidInput(
                "series_id is required for update".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_series_get(series_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut series = self.series_get(series_id).await?;
        if let Some(obj) = series.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/series/{series_id}"), &series)
            .await
    }

    // ---- episodes ----

    /// List episodes (`GET /episode?seriesId=&seasonNumber=`).
    pub async fn episodes_list(
        &self,
        series_id: i64,
        season_number: Option<i64>,
    ) -> Result<Value, ArrError> {
        if series_id <= 0 {
            return Err(ArrError::InvalidInput(
                "series_id is required for list".to_string(),
            ));
        }
        if self.demo {
            return Ok(demo_episodes_list(series_id, season_number));
        }
        let mut query = vec![("seriesId".to_string(), series_id.to_string())];
        if let Some(season) = season_number {
            query.push(("seasonNumber".to_string(), season.to_string()));
        }
        self.get_json("/episode", &query).await
    }

    /// Get one episode (`GET /episode/{id}`).
    pub async fn episode_get(&self, episode_id: i64) -> Result<Value, ArrError> {
        if episode_id <= 0 {
            return Err(ArrError::InvalidInput(
                "episode_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_episode_get(episode_id);
        }
        self.get_json(&format!("/episode/{episode_id}"), &[]).await
    }

    /// Search one episode (`EpisodeSearch`) or a whole season (`SeasonSearch`),
    /// with the legacy ID-verification guards. Demo echoes the command.
    pub async fn episode_search(
        &self,
        episode_id: Option<i64>,
        series_id: Option<i64>,
        season_number: Option<i64>,
    ) -> Result<Value, ArrError> {
        match (episode_id, series_id, season_number) {
            (Some(ep_id), None, None) if ep_id > 0 => {
                let episode = self.episode_get(ep_id).await?;
                let returned = episode.get("id").and_then(|v| v.as_i64()).unwrap_or(-1);
                if returned != ep_id {
                    return Err(ArrError::Protocol(format!(
                        "mismatched episode ID for requested episode {ep_id}; search was not started"
                    )));
                }
                if self.demo {
                    return Ok(serde_json::json!({"name": "EpisodeSearch", "episodeIds": [ep_id]}));
                }
                self.post_json(
                    "/command",
                    &serde_json::json!({"name": "EpisodeSearch", "episodeIds": [ep_id]}),
                )
                .await
            }
            (None, Some(s_id), Some(season)) if s_id > 0 && season >= 0 => {
                let series = self.series_get(s_id).await?;
                let returned = series.get("id").and_then(|v| v.as_i64()).unwrap_or(-1);
                if returned != s_id {
                    return Err(ArrError::Protocol(format!(
                        "mismatched series ID for requested series {s_id}; search was not started"
                    )));
                }
                if self.demo {
                    // Fixture series carries seasons [0, 1]; reject anything else like legacy.
                    let known = [0_i64, 1];
                    if !known.contains(&season) {
                        return Err(ArrError::InvalidInput(format!(
                            "season {season} does not exist in series {s_id}; search was not started"
                        )));
                    }
                    return Ok(serde_json::json!({
                        "name": "SeasonSearch", "seriesId": s_id, "seasonNumber": season
                    }));
                }
                self.post_json(
                    "/command",
                    &serde_json::json!({
                        "name": "SeasonSearch", "seriesId": s_id, "seasonNumber": season
                    }),
                )
                .await
            }
            _ => Err(ArrError::InvalidInput(
                "search requires exactly one positive episode_id, or a positive series_id with a non-negative season_number".to_string(),
            )),
        }
    }

    /// Set monitored flag (GET then PUT, legacy pattern).
    pub async fn episode_set_monitored(
        &self,
        episode_id: i64,
        monitored: bool,
    ) -> Result<Value, ArrError> {
        if episode_id <= 0 {
            return Err(ArrError::InvalidInput(
                "episode_id is required for set_monitored".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_episode_get(episode_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut episode = self.episode_get(episode_id).await?;
        if let Some(obj) = episode.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/episode/{episode_id}"), &episode)
            .await
    }
}

// ---- demo fixtures (offline, deterministic) ----

fn demo_series_list() -> Value {
    serde_json::json!([
        {"id": 42, "title": "Demo Series", "tvdbId": 81189, "monitored": true,
         "seasons": [{"seasonNumber": 0}, {"seasonNumber": 1}]},
        {"id": 43, "title": "Another Show", "tvdbId": 12345, "monitored": false,
         "seasons": [{"seasonNumber": 1}]},
    ])
}

fn demo_series_get(series_id: i64) -> Result<Value, ArrError> {
    let all = demo_series_list();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(series_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: series {series_id} not found"
    )))
}

fn demo_episodes_list(series_id: i64, season_number: Option<i64>) -> Value {
    let all = serde_json::json!([
        {"id": 456, "seriesId": 42, "seasonNumber": 1, "episodeNumber": 1,
         "title": "Pilot", "monitored": true},
        {"id": 457, "seriesId": 42, "seasonNumber": 1, "episodeNumber": 2,
         "title": "Second", "monitored": true},
    ]);
    let items = all
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|e| {
            let sid = e.get("seriesId").and_then(|v| v.as_i64()).unwrap_or(-1);
            if sid != series_id {
                return false;
            }
            match season_number {
                Some(season) => e.get("seasonNumber").and_then(|v| v.as_i64()) == Some(season),
                None => true,
            }
        });
    Value::Array(items.collect())
}

fn demo_episode_get(episode_id: i64) -> Result<Value, ArrError> {
    for candidate in [456_i64, 457] {
        if candidate == episode_id {
            let list = demo_episodes_list(42, None);
            let items = list.as_array().cloned().unwrap_or_default();
            for item in items {
                if item.get("id").and_then(|v| v.as_i64()) == Some(episode_id) {
                    return Ok(item);
                }
            }
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: episode {episode_id} not found"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(SonarrClient::new("", "key").is_err());
        assert!(SonarrClient::new("http://x:8989", "").is_err());
    }

    #[tokio::test]
    async fn demo_series_list_has_two() {
        let client = SonarrClient::demo();
        let list = client.series_list().await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
    }

    #[tokio::test]
    async fn demo_lookup_filters() {
        let client = SonarrClient::demo();
        let hits = client.series_lookup("demo").await.unwrap_or(Value::Null);
        assert_eq!(hits.as_array().map(Vec::len), Some(1));
        let none = client.series_lookup("").await;
        assert!(none.is_err());
    }

    #[tokio::test]
    async fn demo_search_verifies_id() {
        let client = SonarrClient::demo();
        let cmd = client.series_search(42).await.unwrap_or(Value::Null);
        assert_eq!(
            cmd.get("name").and_then(|v| v.as_str()),
            Some("SeriesSearch")
        );
        assert!(client.series_search(0).await.is_err());
        assert!(client.series_search(999).await.is_err());
    }

    #[tokio::test]
    async fn demo_episode_search_branches() {
        let client = SonarrClient::demo();
        let one = client
            .episode_search(Some(456), None, None)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(
            one.get("name").and_then(|v| v.as_str()),
            Some("EpisodeSearch")
        );
        let season = client
            .episode_search(None, Some(42), Some(1))
            .await
            .unwrap_or(Value::Null);
        assert_eq!(
            season.get("name").and_then(|v| v.as_str()),
            Some("SeasonSearch")
        );
        assert!(client
            .episode_search(None, Some(42), Some(9))
            .await
            .is_err());
        assert!(client.episode_search(None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn demo_monitored_round_trip() {
        let client = SonarrClient::demo();
        let updated = client
            .episode_set_monitored(456, false)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(updated.get("monitored"), Some(&Value::Bool(false)));
    }
}
