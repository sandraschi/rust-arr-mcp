//! L1 Readarr driver: authors + books + mutations + add-options over Readarr
//! API v1 (`/api/v1`, `X-Api-Key`, 30 s timeout, 32 MiB cap). Same guard pattern
//! as `sonarr.rs`. `ReadarrClient::demo()` is offline with fixtures.
//! Note: `BookSearch` command shape (`{name, bookIds}`) is reconstructed from
//! the Servarr convention (Movies/Series/EpisodeSearch) — pending live verify.

use std::time::Duration;

use serde_json::{Map, Value};

use crate::error::ArrError;
use crate::sonarr::{MAX_BODY_BYTES, REQUEST_TIMEOUT_SECS};

/// Readarr API path (legacy `READARR_API_PATH`).
pub const API_PATH: &str = "/api/v1";

/// Readarr HTTP client. `demo = true` serves fixtures, never touches the network.
#[derive(Debug, Clone)]
pub struct ReadarrClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
    demo: bool,
}

impl ReadarrClient {
    /// Live client. Empty URL or key is rejected.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArrError> {
        if base_url.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "readarr base_url is empty".to_string(),
            ));
        }
        if api_key.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "readarr api_key is empty".to_string(),
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
                "readarr {context}: timed out after {REQUEST_TIMEOUT_SECS}s"
            ));
        }
        if err.is_connect() {
            return ArrError::Offline(format!(
                "readarr {context}: unreachable ({})",
                self.base_url
            ));
        }
        if let Some(status) = err.status() {
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return ArrError::Auth(format!(
                    "readarr {context}: rejected credentials (401/403)"
                ));
            }
            return ArrError::Protocol(format!("readarr {context}: HTTP {status}"));
        }
        ArrError::Protocol(format!("readarr {context}: {err}"))
    }

    async fn check_body(&self, context: &str, bytes: &[u8]) -> Result<(), ArrError> {
        if bytes.len() > MAX_BODY_BYTES {
            return Err(ArrError::Protocol(format!(
                "readarr {context}: response {} bytes exceeds {MAX_BODY_BYTES} cap",
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
                    "readarr: rejected credentials (401/403)".to_string(),
                ));
            }
            return Err(ArrError::Protocol(format!(
                "readarr GET {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("readarr GET {path}: invalid JSON: {e}")))
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
                "readarr POST {path}: HTTP {status}"
            )));
        }
        if bytes.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("readarr POST {path}: invalid JSON: {e}")))
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
                "readarr PUT {path}: HTTP {status}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|e| ArrError::Protocol(format!("readarr PUT {path}: invalid JSON: {e}")))
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
                "readarr DELETE {path}: HTTP {status}"
            )));
        }
        Ok(())
    }

    // ---- authors ----

    /// List authors (`GET /author`). Demo returns two fixtures.
    pub async fn authors_list(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(demo_authors_list());
        }
        self.get_json("/author", &[]).await
    }

    /// Lookup metadata by term (`GET /author/lookup?term=`). Demo filters fixtures.
    pub async fn authors_lookup(&self, term: &str) -> Result<Value, ArrError> {
        if term.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "term is required for lookup".to_string(),
            ));
        }
        if self.demo {
            let needle = term.to_ascii_lowercase();
            let all = demo_authors_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|a| {
                    a.get("authorName")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_ascii_lowercase().contains(&needle))
                        .unwrap_or(false)
                });
            return Ok(Value::Array(items.collect()));
        }
        self.get_json("/author/lookup", &[("term".to_string(), term.to_string())])
            .await
    }

    /// Get one author (`GET /author/{id}`).
    pub async fn author_get(&self, author_id: i64) -> Result<Value, ArrError> {
        if author_id <= 0 {
            return Err(ArrError::InvalidInput(
                "author_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_author_get(author_id);
        }
        self.get_json(&format!("/author/{author_id}"), &[]).await
    }

    /// Add an author (`POST /author` with `foreignAuthorId`). Demo echoes.
    pub async fn author_add(
        &self,
        foreign_author_id: &str,
        quality_profile_id: i64,
        root_folder_path: &str,
        monitored: bool,
    ) -> Result<Value, ArrError> {
        if foreign_author_id.trim().is_empty()
            || quality_profile_id <= 0
            || root_folder_path.trim().is_empty()
        {
            return Err(ArrError::InvalidInput(
                "foreign_author_id, quality_profile_id, and root_folder_path are required"
                    .to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({
                "id": 9004,
                "foreignAuthorId": foreign_author_id,
                "authorName": "Demo Author",
                "qualityProfileId": quality_profile_id,
                "rootFolderPath": root_folder_path,
                "monitored": monitored,
            }));
        }
        let body = serde_json::json!({
            "foreignAuthorId": foreign_author_id,
            "qualityProfileId": quality_profile_id,
            "rootFolderPath": root_folder_path,
            "monitored": monitored,
        });
        self.post_json("/author", &body).await
    }

    /// Delete an author (`DELETE /author/{id}`). Demo acknowledges.
    pub async fn author_delete(
        &self,
        author_id: i64,
        delete_files: bool,
    ) -> Result<Value, ArrError> {
        if author_id <= 0 {
            return Err(ArrError::InvalidInput(
                "author_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": author_id}));
        }
        self.delete(
            &format!("/author/{author_id}"),
            &[("deleteFiles".to_string(), delete_files.to_string())],
        )
        .await?;
        Ok(serde_json::json!({"deleted": author_id}))
    }

    /// Update monitored flag (GET then PUT, legacy pattern).
    pub async fn author_update(&self, author_id: i64, monitored: bool) -> Result<Value, ArrError> {
        if author_id <= 0 {
            return Err(ArrError::InvalidInput(
                "author_id is required for update".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_author_get(author_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut author = self.author_get(author_id).await?;
        if let Some(obj) = author.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/author/{author_id}"), &author)
            .await
    }

    // ---- books ----

    /// List books (`GET /book`, optional `authorId` filter).
    pub async fn books_list(&self, author_id: Option<i64>) -> Result<Value, ArrError> {
        if self.demo {
            let all = demo_books_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|b| match author_id {
                    Some(want) => b.get("authorId").and_then(|v| v.as_i64()) == Some(want),
                    None => true,
                });
            return Ok(Value::Array(items.collect()));
        }
        let mut query = Vec::new();
        if let Some(author_id) = author_id {
            query.push(("authorId".to_string(), author_id.to_string()));
        }
        self.get_json("/book", &query).await
    }

    /// Get one book (`GET /book/{id}`).
    pub async fn book_get(&self, book_id: i64) -> Result<Value, ArrError> {
        if book_id <= 0 {
            return Err(ArrError::InvalidInput(
                "book_id is required for get".to_string(),
            ));
        }
        if self.demo {
            return demo_book_get(book_id);
        }
        self.get_json(&format!("/book/{book_id}"), &[]).await
    }

    /// Lookup metadata by term (`GET /book/lookup?term=`).
    pub async fn books_lookup(&self, term: &str) -> Result<Value, ArrError> {
        if term.trim().is_empty() {
            return Err(ArrError::InvalidInput(
                "term is required for lookup".to_string(),
            ));
        }
        if self.demo {
            let needle = term.to_ascii_lowercase();
            let all = demo_books_list();
            let items = all
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|b| {
                    b.get("title")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_ascii_lowercase().contains(&needle))
                        .unwrap_or(false)
                });
            return Ok(Value::Array(items.collect()));
        }
        self.get_json("/book/lookup", &[("term".to_string(), term.to_string())])
            .await
    }

    /// Native `BookSearch` for one book (id-verified; reconstructed Servarr shape).
    pub async fn book_search(&self, book_id: i64) -> Result<Value, ArrError> {
        if book_id <= 0 {
            return Err(ArrError::InvalidInput(
                "a positive book_id is required for search".to_string(),
            ));
        }
        let book = self.book_get(book_id).await?;
        let returned = book.get("id").and_then(|v| v.as_i64()).unwrap_or(-1);
        if returned != book_id {
            return Err(ArrError::Protocol(format!(
                "mismatched book ID for requested book {book_id}; search was not started"
            )));
        }
        if self.demo {
            return Ok(serde_json::json!({"name": "BookSearch", "bookIds": [book_id]}));
        }
        self.post_json(
            "/command",
            &serde_json::json!({"name": "BookSearch", "bookIds": [book_id]}),
        )
        .await
    }

    /// Set monitored flag (GET then PUT, legacy pattern).
    pub async fn book_set_monitored(
        &self,
        book_id: i64,
        monitored: bool,
    ) -> Result<Value, ArrError> {
        if book_id <= 0 {
            return Err(ArrError::InvalidInput(
                "book_id is required for set_monitored".to_string(),
            ));
        }
        if self.demo {
            let mut rec = demo_book_get(book_id)?;
            if let Some(obj) = rec.as_object_mut() {
                obj.insert("monitored".to_string(), Value::Bool(monitored));
            }
            return Ok(rec);
        }
        let mut book = self.book_get(book_id).await?;
        if let Some(obj) = book.as_object_mut() {
            obj.insert("monitored".to_string(), Value::Bool(monitored));
        }
        self.put_json(&format!("/book/{book_id}"), &book).await
    }

    // ---- book mutations (write-safety showcase: explicit params, no silent defaults) ----

    /// Add one book (`POST /book` with `foreignBookId`). Demo echoes.
    pub async fn book_add(
        &self,
        foreign_book_id: &str,
        title: &str,
        quality_profile_id: i64,
        root_folder_path: &str,
        monitored: bool,
    ) -> Result<Value, ArrError> {
        if foreign_book_id.trim().is_empty()
            || quality_profile_id <= 0
            || root_folder_path.trim().is_empty()
        {
            return Err(ArrError::InvalidInput(
                "foreign_book_id, quality_profile_id, and root_folder_path are required"
                    .to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({
                "id": 9005,
                "foreignBookId": foreign_book_id,
                "title": title,
                "qualityProfileId": quality_profile_id,
                "rootFolderPath": root_folder_path,
                "monitored": monitored,
            }));
        }
        let body = serde_json::json!({
            "foreignBookId": foreign_book_id,
            "title": title,
            "qualityProfileId": quality_profile_id,
            "rootFolderPath": root_folder_path,
            "monitored": monitored,
        });
        self.post_json("/book", &body).await
    }

    /// Delete one book (`DELETE /book/{id}`). Demo acknowledges.
    pub async fn book_delete(&self, book_id: i64, delete_files: bool) -> Result<Value, ArrError> {
        if book_id <= 0 {
            return Err(ArrError::InvalidInput(
                "book_id is required for delete".to_string(),
            ));
        }
        if self.demo {
            return Ok(serde_json::json!({"deleted": book_id}));
        }
        self.delete(
            &format!("/book/{book_id}"),
            &[("deleteFiles".to_string(), delete_files.to_string())],
        )
        .await?;
        Ok(serde_json::json!({"deleted": book_id}))
    }

    /// Delete many books (sequential; stops are per-item errors collected, never panics).
    pub async fn books_delete_many(
        &self,
        book_ids: &[i64],
        delete_files: bool,
    ) -> Result<Value, ArrError> {
        if book_ids.is_empty() {
            return Err(ArrError::InvalidInput(
                "at least one book_id is required".to_string(),
            ));
        }
        let mut deleted = Vec::new();
        let mut failed = Vec::new();
        for id in book_ids {
            match self.book_delete(*id, delete_files).await {
                Ok(_) => deleted.push(*id),
                Err(e) => failed.push(serde_json::json!({"id": id, "error": e.to_string()})),
            }
        }
        Ok(serde_json::json!({"deleted": deleted, "failed": failed}))
    }

    /// Add-options lookup: root folders + quality/metadata profiles + tags (4 GETs).
    pub async fn add_options(&self) -> Result<Value, ArrError> {
        if self.demo {
            return Ok(serde_json::json!({
                "rootFolders": [{"path": "/books"}],
                "qualityProfiles": [{"id": 1, "name": "Any"}],
                "metadataProfiles": [{"id": 1, "name": "Standard"}],
                "tags": [],
            }));
        }
        let root_folders = self
            .get_json("/rootfolder", &[])
            .await
            .unwrap_or(Value::Null);
        let quality = self
            .get_json("/qualityprofile", &[])
            .await
            .unwrap_or(Value::Null);
        let metadata = self
            .get_json("/metadataprofile", &[])
            .await
            .unwrap_or(Value::Null);
        let tags = self.get_json("/tag", &[]).await.unwrap_or(Value::Null);
        Ok(serde_json::json!({
            "rootFolders": root_folders,
            "qualityProfiles": quality,
            "metadataProfiles": metadata,
            "tags": tags,
        }))
    }
}

// ---- demo fixtures (offline, deterministic) ----

fn demo_authors_list() -> Value {
    serde_json::json!([
        {"id": 31, "authorName": "Demo Author", "foreignAuthorId": "gr-demo-1", "monitored": true},
        {"id": 32, "authorName": "Another Writer", "foreignAuthorId": "gr-demo-2", "monitored": false},
    ])
}

fn demo_author_get(author_id: i64) -> Result<Value, ArrError> {
    let all = demo_authors_list();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(author_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: author {author_id} not found"
    )))
}

fn demo_books_list() -> Value {
    serde_json::json!([
        {"id": 61, "authorId": 31, "title": "Demo Book", "monitored": true},
        {"id": 62, "authorId": 31, "title": "Second Volume", "monitored": true},
    ])
}

fn demo_book_get(book_id: i64) -> Result<Value, ArrError> {
    let all = demo_books_list();
    let items = all.as_array().cloned().unwrap_or_default();
    for item in items {
        if item.get("id").and_then(|v| v.as_i64()) == Some(book_id) {
            return Ok(item);
        }
    }
    Err(ArrError::Protocol(format!(
        "demo: book {book_id} not found"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_config() {
        assert!(ReadarrClient::new("", "key").is_err());
        assert!(ReadarrClient::new("http://x:8787", "").is_err());
    }

    #[tokio::test]
    async fn demo_authors_crud() {
        let client = ReadarrClient::demo();
        let list = client.authors_list().await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        assert!(client.authors_lookup("").await.is_err());
        let added = client
            .author_add("gr-new", 1, "/books", true)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(
            added.get("foreignAuthorId").and_then(|v| v.as_str()),
            Some("gr-new")
        );
        let updated = client.author_update(31, false).await.unwrap_or(Value::Null);
        assert_eq!(updated.get("monitored"), Some(&Value::Bool(false)));
        let deleted = client.author_delete(31, false).await.unwrap_or(Value::Null);
        assert_eq!(deleted.get("deleted").and_then(|v| v.as_i64()), Some(31));
    }

    #[tokio::test]
    async fn demo_books_search_and_mutations() {
        let client = ReadarrClient::demo();
        let list = client.books_list(Some(31)).await.unwrap_or(Value::Null);
        assert_eq!(list.as_array().map(Vec::len), Some(2));
        let cmd = client.book_search(61).await.unwrap_or(Value::Null);
        assert_eq!(cmd.get("name").and_then(|v| v.as_str()), Some("BookSearch"));
        assert!(client.book_search(999).await.is_err());
        let added = client
            .book_add("gr-book-new", "New Book", 1, "/books", true)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(added.get("id").and_then(|v| v.as_i64()), Some(9005));
        let many = client
            .books_delete_many(&[61, 62], false)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(
            many.get("deleted").and_then(|v| v.as_array()).map(Vec::len),
            Some(2)
        );
        assert!(client.books_delete_many(&[], false).await.is_err());
        let opts = client.add_options().await.unwrap_or(Value::Null);
        assert!(opts.get("rootFolders").is_some());
    }
}
