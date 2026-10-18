//! L1 Lidarr driver: artists + albums over Lidarr API v1 (`/api/v1`,
//! `X-Api-Key`, 30 s timeout, 32 MiB cap). Same guard pattern as `sonarr.rs`.
//! `LidarrClient::demo()` is offline with deterministic fixtures.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;
use crate::sonarr::{MAX_BODY_BYTES, REQUEST_TIMEOUT_SECS};

/// Lidarr API path (legacy `LIDARR_API_PATH`).
pub const API_PATH: &str = "/api/v1";

/// Lidarr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct LidarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl LidarrClient {
    /// Live client. Empty URL or key is rejected.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "lidarr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "lidarr api_key is empty".to_string(),
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
                "lidarr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!("lidarr {context}: unreachable ({})", self.base_url));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!("lidarr {context}: rejected credentials (401/403)"));
            }
            return ArrError::Protocol(format!("lidarr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("lidarr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "lidarr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
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
                    "lidarr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "lidarr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("lidarr GET {path}: invalid JSON: {e}")))
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
                "lidarr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("lidarr POST {path}: invalid JSON: {e}")))
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
                "lidarr PUT {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("lidarr PUT {path}: invalid JSON: {e}")))
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
                "lidarr DELETE {path}: HTTP {status}"
            )));
        }
        Ok(())
    }

    // ---- artists ----

    /// List artists (`GET /artist`, optional `mbId` filter). Demo returns two fixtures.
    pub async fn artists_list(&self, mb_id: Option<&str>) -> Result<Value, ArrError> {
        if self.demo {
            let all = demo_artists_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|a| match mb_id {
                    Some(want) => a.get("foreignArtistId").and_then(|v| v.as_str()) == Some(want),
                    None => true,
                });
            return Ok(Value::Array(items.collect()));
        }
        let mut query = Vec::new();
        if let Some(mb_id) = mb_id {
            if !mb_id.trim().is_empty() {
                query.push(("mbId".to_string(), mb_id.to_string()));
            }
        }
        self.get_json("/artist", &query).await
    }

    /// Lookup metadata by term (`GET /artist/lookup?term=`). Demo filters fixtures.
    pub async fn artists_lookup(&self, term: &str) -> Result<Value, ArrError> {
        if term.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "term is required for lookup".to_string(),
            ));
        }
        if self.demo {
            let needle = term.to_ascii_lowercase();
            let all = demo_artists_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|a| {
                    a.get("artistName")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_ascii_lowercase().contains(&needle))
                        .unwrap_or(false)
                });
            return Ok(Value::Array(items.collect()));
        }
        self.get_json("/artist/lookup", &[("term".to_string(), term.to_string())])
            .await
    }

    /// Get one artist (`GET /artist/{id}`).
    pub async fn artist_get(&self, artist_id: i64) -> Result<Value, ArrError> {
        if artist_id <= 0 {
            return Err(ArrError::InvalidInput(
                "artist_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_artist_get(artist_id);
        }
        self.get_json(&format!("/artist/{artist_id}"), &[]).await
    }

    /// Add an artist (`POST /artist` with `foreignArtistId`). Demo echoes.
    pub async fn artist_add(
        &self,
        foreign_artist_id: &str,
        quality_profile_id: i64,
        root_folder_path: &str,
        monitored: bool,
    ) -> Result<Value, ArrError> {
        if foreign_artist_id.trim().is_empty()
            || quality_profile_id <= 0
            || root_folder_path.trim().is_empty()
        {
            return Err(ArrError::InvalidInput(
                "foreign_artist_id, quality_profile_id, and root_folder_path are required"
                    .to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({
                "id": 9003,
                "foreignArtistId": foreign_artist_id,
                "artistName": "Demo Artist",
                "qualityProfileId": quality_profile_id,
                "rootFolderPath": root_folder_path,
                "monitored": monitored,
            }));
        }
        let body = serde_json::json!({
            "foreignArtistId": foreign_artist_id,
            "qualityProfileId": quality_profile_id,
            "rootFolderPath": root_folder_path,
            "monitored": monitored,
        });
        self.post_json("/artist", &body).await
    }

    /// Delete an artist (`DELETE /artist/{id}`). Demo acknowledges.
    pub async fn artist_delete(
        &self,
        artist_id: i64,
        delete_files: bool,
    ) -> Result<Value, ArrError> {
        if artist_id <= 0 {
            return Err(ArrError::InvalidInput(
                "artist_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": artist_id}));
        }
        self.delete(
            &format!("/artist/{artist_id}"),
            &[("deleteFiles".to_string(), delete_files.to_string())],
        )
        .await?;
        Ok(serde_json::json!({"deleted": artist_id}))
    }

    /// Update monitored flag (GET then PUT, legacy pattern).
    pub async fn artist_update(&self, artist_id: i64, monitored: bool) -> Result<Value, ArrError> {
        if artist_id <= 0 {
            return Err(ArrError::InvalidInput(
                "artist_id is required for update".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_artist_get(artist_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut artist = self.artist_get(artist_id).await?;
        if let Some(obj) = artist.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/artist/{artist_id}"), &artist)
            .await
    }

    // ---- albums ----

    /// List albums (`GET /album`, optional `artistId` filter).
    pub async fn albums_list(&self, artist_id: Option<i64>) -> Result<Value, ArrError> {
        if self.demo {
            let all = demo_albums_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|a| match artist_id {
                    Some(want) => a.get("artistId").and_then(|v| v.as_i64()) == Some(want),
                    None => true,
                });
            return Ok(Value::Array(items.collect()));
        }
        let mut query = Vec::new();
        if let Some(artist_id) = artist_id {
            query.push(("artistId".to_string(), artist_id.to_string()));
        }
        self.get_json("/album", &query).await
    }

    /// Get one album (`GET /album/{id}`).
    pub async fn album_get(&self, album_id: i64) -> Result<Value, ArrError> {
        if album_id <= 0 {
            return Err(ArrError::InvalidInput(
                "album_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_album_get(album_id);
        }
        self.get_json(&format!("/album/{album_id}"), &[]).await
    }

    /// Lookup metadata by term (`GET /album/lookup?term=`).
    pub async fn albums_lookup(&self, term: &str) -> Result<Value, ArrError> {
        if term.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "term is required for lookup".to_string(),
            ));
        }
        if self.demo {
            let needle = term.to_ascii_lowercase();
            let all = demo_albums_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|a| {
                    a.get("title")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_ascii_lowercase().contains(&needle))
                        .unwrap_or(false)
                });
            return Ok(Value::Array(items.collect()));
        }
        self.get_json("/album/lookup", &[("term".to_string(), term.to_string())])
            .await
    }

    /// Set monitored flag (GET then PUT, legacy pattern).
    pub async fn album_set_monitored(
        &self,
        album_id: i64,
        monitored: bool,
    ) -> Result<Value, ArrError> {
        if album_id <= 0 {
            return Err(ArrError::InvalidInput(
                "album_id is required for set_monitored".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_album_get(album_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut album = self.album_get(album_id).await?;
        if let Some(obj) = album.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/album/{album_id}"), &album).await
    }
}

// ---- demo fixtures (offline, deterministic) ----

fn demo_artists_list() -> Value {
    serde_json::json!([
        {"id": 11, "artistName": "Demo Artist", "foreignArtistId": "mb-demo-1", "monitored": true},
        {"id": 12, "artistName": "Another Act", "foreignArtistId": "mb-demo-2", "monitored": false},
    ])
}

fn demo_artist_get(artist_id: i64) -> Result<Value, ArrError> {
    let all = demo_artists_list();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(artist_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: artist {artist_id} not found"
    )))
}

fn demo_albums_list() -> Value {
    serde_json::json!([
        {"id": 21, "artistId": 11, "title": "Demo Record", "monitored": true},
        {"id": 22, "artistId": 11, "title": "Second Pressing", "monitored": true},
    ])
}

fn demo_album_get(album_id: i64) -> Result<Value, ArrError> {
    let all = demo_albums_list();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(album_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: album {album_id} not found"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(LidarrClient::new("", "key").is_err());
        assert!(LidarrClient::new("http://x:8686", "").is_err());
    }

    #[tokio::test]
    async fn demo_artists_crud() {
        let client = LidarrClient::demo();
        let list = client.artists_list(None).await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        assert!(client.artists_lookup("").await.is_err());
        let added = client
            .artist_add("mb-new", 1, "/music", true)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(
            added.get("foreignArtistId").and_then(|v| v.as_str()),
            Some("mb-new")
        );
        assert!(client.artist_add("", 1, "/music", true).await.is_err());
        let updated = client.artist_update(11, false).await.unwrap_or(Value::Null);
        assert_eq!(updated.get("monitored"), Some(&Value::Bool(false)));
        let deleted = client.artist_delete(11, false).await.unwrap_or(Value::Null);
        assert_eq!(deleted.get("deleted").and_then(|v| v.as_i64()), Some(11));
    }

    #[tokio::test]
    async fn demo_albums_monitored_round_trip() {
        let client = LidarrClient::demo();
        let list = client.albums_list(Some(11)).await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        let updated = client
            .album_set_monitored(21, false)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(updated.get("monitored"), Some(&Value::Bool(false)));
        assert!(client.album_set_monitored(0, true).await.is_err());
    }
}
