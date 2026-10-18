//! Command bus: 1:1 typed variants for every legacy Sonarr op + `Status`.
//! `dispatch` runs against an explicit client; `dispatch_auto` resolves
//! config (explicit env → imported `../arr-mcp` → Demo fallback).
//! More services (radarr/lidarr/…) gain variants here as their drivers land.

use serde_json::Value;

use crate::error::ArrError;
use crate::import::{discover_legacy_dir, load_from_dir, ArrServices};
use crate::radarr::RadarrClient;
use crate::sonarr::SonarrClient;

/// All commands (Phase 2b: `Status` + Sonarr 7+4 ops).
#[derive(Debug, Clone)]
pub enum Command {
    Status,
    SonarrSeriesList,
    SonarrSeriesLookup {
        term: String,
    },
    SonarrSeriesGet {
        series_id: i64,
    },
    SonarrSeriesAdd {
        tvdb_id: i64,
        title: String,
        quality_profile_id: i64,
        root_folder_path: String,
        monitored: bool,
        search_for_missing: bool,
    },
    SonarrSeriesSearch {
        series_id: i64,
    },
    SonarrSeriesDelete {
        series_id: i64,
        delete_files: bool,
        add_exclusion: bool,
    },
    SonarrSeriesUpdate {
        series_id: i64,
        monitored: bool,
    },
    SonarrEpisodesList {
        series_id: i64,
        season_number: Option<i64>,
    },
    SonarrEpisodeGet {
        episode_id: i64,
    },
    SonarrEpisodeSearch {
        episode_id: Option<i64>,
        series_id: Option<i64>,
        season_number: Option<i64>,
    },
    SonarrEpisodeSetMonitored {
        episode_id: i64,
        monitored: bool,
    },
    RadarrMoviesList,
    RadarrMoviesLookup {
        term: String,
    },
    RadarrMovieGet {
        movie_id: i64,
    },
    RadarrMovieAdd {
        tmdb_id: i64,
        title: String,
        quality_profile_id: i64,
        root_folder_path: String,
        monitored: bool,
        search_for_movie: bool,
    },
    RadarrMovieSearch {
        movie_id: i64,
    },
    RadarrMovieDelete {
        movie_id: i64,
        delete_files: bool,
        add_exclusion: bool,
    },
    RadarrMovieUpdate {
        movie_id: i64,
        monitored: bool,
    },
    RadarrMoviesImport {
        folder: Option<String>,
    },
}

/// Run one command against an explicit Sonarr client.
pub async fn dispatch(client: &SonarrClient, command: &Command) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2c-radarr",
            "ok": true,
        })),
        Command::SonarrSeriesList => client.series_list().await,
        Command::SonarrSeriesLookup { term } => client.series_lookup(term).await,
        Command::SonarrSeriesGet { series_id } => client.series_get(*series_id).await,
        Command::SonarrSeriesAdd {
            tvdb_id,
            title,
            quality_profile_id,
            root_folder_path,
            monitored,
            search_for_missing,
        } => {
            client
                .series_add(
                    *tvdb_id,
                    title,
                    *quality_profile_id,
                    root_folder_path,
                    *monitored,
                    *search_for_missing,
                )
                .await
        }
        Command::SonarrSeriesSearch { series_id } => client.series_search(*series_id).await,
        Command::SonarrSeriesDelete {
            series_id,
            delete_files,
            add_exclusion,
        } => {
            client
                .series_delete(*series_id, *delete_files, *add_exclusion)
                .await
        }
        Command::SonarrSeriesUpdate {
            series_id,
            monitored,
        } => client.series_update(*series_id, *monitored).await,
        Command::SonarrEpisodesList {
            series_id,
            season_number,
        } => client.episodes_list(*series_id, *season_number).await,
        Command::SonarrEpisodeGet { episode_id } => client.episode_get(*episode_id).await,
        Command::SonarrEpisodeSearch {
            episode_id,
            series_id,
            season_number,
        } => {
            client
                .episode_search(*episode_id, *series_id, *season_number)
                .await
        }
        Command::SonarrEpisodeSetMonitored {
            episode_id,
            monitored,
        } => client.episode_set_monitored(*episode_id, *monitored).await,
        Command::RadarrMoviesList
        | Command::RadarrMoviesLookup { .. }
        | Command::RadarrMovieGet { .. }
        | Command::RadarrMovieAdd { .. }
        | Command::RadarrMovieSearch { .. }
        | Command::RadarrMovieDelete { .. }
        | Command::RadarrMovieUpdate { .. }
        | Command::RadarrMoviesImport { .. } => Err(ArrError::Unsupported(
            "radarr command requires a Radarr client: use dispatch_radarr".to_string(),
        )),
    }
}

/// Run one command against an explicit Radarr client.
pub async fn dispatch_radarr(client: &RadarrClient, command: &Command) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2c-radarr",
            "ok": true,
        })),
        Command::RadarrMoviesList => client.movies_list().await,
        Command::RadarrMoviesLookup { term } => client.movies_lookup(term).await,
        Command::RadarrMovieGet { movie_id } => client.movie_get(*movie_id).await,
        Command::RadarrMovieAdd {
            tmdb_id,
            title,
            quality_profile_id,
            root_folder_path,
            monitored,
            search_for_movie,
        } => {
            client
                .movie_add(
                    *tmdb_id,
                    title,
                    *quality_profile_id,
                    root_folder_path,
                    *monitored,
                    *search_for_movie,
                )
                .await
        }
        Command::RadarrMovieSearch { movie_id } => client.movie_search(*movie_id).await,
        Command::RadarrMovieDelete {
            movie_id,
            delete_files,
            add_exclusion,
        } => {
            client
                .movie_delete(*movie_id, *delete_files, *add_exclusion)
                .await
        }
        Command::RadarrMovieUpdate {
            movie_id,
            monitored,
        } => client.movie_update(*movie_id, *monitored).await,
        Command::RadarrMoviesImport { folder } => client.movies_import(folder.as_deref()).await,
        _ => Err(ArrError::Unsupported(
            "sonarr command requires a Sonarr client: use dispatch".to_string(),
        )),
    }
}

/// Resolve the right service (process env → imported legacy checkout → Demo)
/// from the command variant and run it. Never panics.
pub async fn dispatch_auto(command: Command) -> Result<Value, ArrError> {
    match &command {
        Command::RadarrMoviesList
        | Command::RadarrMoviesLookup { .. }
        | Command::RadarrMovieGet { .. }
        | Command::RadarrMovieAdd { .. }
        | Command::RadarrMovieSearch { .. }
        | Command::RadarrMovieDelete { .. }
        | Command::RadarrMovieUpdate { .. }
        | Command::RadarrMoviesImport { .. } => {
            let client = resolve_radarr_client();
            dispatch_radarr(&client, &command).await
        }
        _ => {
            let client = resolve_sonarr_client();
            dispatch(&client, &command).await
        }
    }
}

fn resolve_sonarr_client() -> SonarrClient {
    // 1. Explicit process env.
    let env_url = std::env::var("SONARR_URL").ok().filter(|v| !v.is_empty());
    let env_key = std::env::var("SONARR_API_KEY")
        .ok()
        .filter(|v| !v.is_empty());
    if let (Some(url), Some(key)) = (env_url, env_key) {
        if let Ok(client) = SonarrClient::new(&url, &key) {
            return client;
        }
    }
    // 2. Imported legacy checkout.
    if let Some(dir) = discover_legacy_dir() {
        if let Ok(services) = load_from_dir(&dir) {
            let imported: ArrServices = services;
            if let Some(cfg) = imported.get("sonarr") {
                if let (Some(url), Some(key)) = (cfg.url.clone(), cfg.api_key.clone()) {
                    if let Ok(client) = SonarrClient::new(&url, &key) {
                        return client;
                    }
                }
            }
        }
    }
    // 3. Demo fallback (offline, deterministic).
    SonarrClient::demo()
}

fn resolve_radarr_client() -> RadarrClient {
    // 1. Explicit process env.
    let env_url = std::env::var("RADARR_URL").ok().filter(|v| !v.is_empty());
    let env_key = std::env::var("RADARR_API_KEY")
        .ok()
        .filter(|v| !v.is_empty());
    if let (Some(url), Some(key)) = (env_url, env_key) {
        if let Ok(client) = RadarrClient::new(&url, &key) {
            return client;
        }
    }
    // 2. Imported legacy checkout.
    if let Some(dir) = discover_legacy_dir() {
        if let Ok(services) = load_from_dir(&dir) {
            let imported: ArrServices = services;
            if let Some(cfg) = imported.get("radarr") {
                if let (Some(url), Some(key)) = (cfg.url.clone(), cfg.api_key.clone()) {
                    if let Ok(client) = RadarrClient::new(&url, &key) {
                        return client;
                    }
                }
            }
        }
    }
    // 3. Demo fallback (offline, deterministic).
    RadarrClient::demo()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_future_is_send() {
        fn assert_send<T: Send>(_: T) {}
        assert_send(dispatch_auto(Command::Status));
    }

    #[tokio::test]
    async fn auto_status_without_config() {
        let v = dispatch_auto(Command::Status).await.unwrap_or(Value::Null);
        assert_eq!(v.get("ok"), Some(&serde_json::Value::Bool(true)));
    }

    #[tokio::test]
    async fn demo_series_list_explicit() {
        use crate::sonarr::SonarrClient;
        let client = SonarrClient::demo();
        let v = dispatch(&client, &Command::SonarrSeriesList)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }

    #[tokio::test]
    async fn demo_movies_list_explicit() {
        use crate::radarr::RadarrClient;
        let client = RadarrClient::demo();
        let v = dispatch_radarr(&client, &Command::RadarrMoviesList)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }
}
