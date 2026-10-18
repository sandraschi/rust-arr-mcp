//! Command bus: 1:1 typed variants for every legacy Sonarr op + `Status`.
//! `dispatch` runs against an explicit client; `dispatch_auto` resolves
//! config (explicit env → imported `../arr-mcp` → Demo fallback).
//! More services (radarr/lidarr/…) gain variants here as their drivers land.

use serde_json::Value;

use crate::bazarr::BazarrClient;
use crate::error::ArrError;
use crate::import::{discover_legacy_dir, load_from_dir, ArrServices};
use crate::lidarr::LidarrClient;
use crate::overseerr::OverseerrClient;
use crate::prowlarr::ProwlarrClient;
use crate::radarr::RadarrClient;
use crate::readarr::ReadarrClient;
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
    LidarrArtistsList {
        mb_id: Option<String>,
    },
    LidarrArtistsLookup {
        term: String,
    },
    LidarrArtistGet {
        artist_id: i64,
    },
    LidarrArtistAdd {
        foreign_artist_id: String,
        quality_profile_id: i64,
        root_folder_path: String,
        monitored: bool,
    },
    LidarrArtistDelete {
        artist_id: i64,
        delete_files: bool,
    },
    LidarrArtistUpdate {
        artist_id: i64,
        monitored: bool,
    },
    LidarrAlbumsList {
        artist_id: Option<i64>,
    },
    LidarrAlbumGet {
        album_id: i64,
    },
    LidarrAlbumsLookup {
        term: String,
    },
    LidarrAlbumSetMonitored {
        album_id: i64,
        monitored: bool,
    },
    ReadarrAuthorsList,
    ReadarrAuthorsLookup {
        term: String,
    },
    ReadarrAuthorGet {
        author_id: i64,
    },
    ReadarrAuthorAdd {
        foreign_author_id: String,
        quality_profile_id: i64,
        root_folder_path: String,
        monitored: bool,
    },
    ReadarrAuthorDelete {
        author_id: i64,
        delete_files: bool,
    },
    ReadarrAuthorUpdate {
        author_id: i64,
        monitored: bool,
    },
    ReadarrBooksList {
        author_id: Option<i64>,
    },
    ReadarrBookGet {
        book_id: i64,
    },
    ReadarrBooksLookup {
        term: String,
    },
    ReadarrBookSearch {
        book_id: i64,
    },
    ReadarrBookSetMonitored {
        book_id: i64,
        monitored: bool,
    },
    ReadarrBookAdd {
        foreign_book_id: String,
        title: String,
        quality_profile_id: i64,
        root_folder_path: String,
        monitored: bool,
    },
    ReadarrBookDelete {
        book_id: i64,
        delete_files: bool,
    },
    ReadarrBooksDeleteMany {
        book_ids: Vec<i64>,
        delete_files: bool,
    },
    ReadarrAddOptions,
    BazarrWanted {
        kind: String,
    },
    BazarrSearch {
        series_id: Option<i64>,
        episode_id: Option<i64>,
        movie_id: Option<i64>,
    },
    BazarrDownload {
        subtitle_id: String,
    },
    BazarrHistory,
    BazarrProviders,
    BazarrLanguages,
    ProwlarrIndexersList,
    ProwlarrIndexerGet {
        indexer_id: i64,
    },
    ProwlarrIndexerAdd {
        fields: serde_json::Value,
    },
    ProwlarrIndexerUpdate {
        indexer_id: i64,
        fields: serde_json::Value,
    },
    ProwlarrIndexerDelete {
        indexer_id: i64,
    },
    ProwlarrIndexerTest {
        indexer_id: i64,
    },
    ProwlarrIndexersTestAll,
    ProwlarrIndexerSchema,
    ProwlarrSearch {
        query: String,
    },
    ProwlarrApplicationsList,
    ProwlarrApplicationGet {
        app_id: i64,
    },
    ProwlarrApplicationSync {
        app_id: i64,
    },
    ProwlarrApplicationsSyncAll,
    ProwlarrApplicationTest {
        app_id: i64,
    },
    ProwlarrHistoryList,
    ProwlarrHistorySince {
        since: String,
    },
    ProwlarrHistoryByIndexer {
        indexer_id: i64,
    },
    OverseerrRequestsList,
    OverseerrRequestGet {
        request_id: i64,
    },
    OverseerrRequestCreate {
        media_type: String,
        media_id: i64,
        seasons: Vec<i64>,
    },
    OverseerrRequestApprove {
        request_id: i64,
    },
    OverseerrRequestDecline {
        request_id: i64,
    },
    OverseerrRequestDelete {
        request_id: i64,
    },
    OverseerrRequestsCount,
    OverseerrRequestsPending,
    OverseerrSearch {
        query: String,
    },
    OverseerrUsersList,
    OverseerrUserGet {
        user_id: i64,
    },
    OverseerrUserRequests {
        user_id: i64,
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
        Command::LidarrArtistsList { .. }
        | Command::LidarrArtistsLookup { .. }
        | Command::LidarrArtistGet { .. }
        | Command::LidarrArtistAdd { .. }
        | Command::LidarrArtistDelete { .. }
        | Command::LidarrArtistUpdate { .. }
        | Command::LidarrAlbumsList { .. }
        | Command::LidarrAlbumGet { .. }
        | Command::LidarrAlbumsLookup { .. }
        | Command::LidarrAlbumSetMonitored { .. } => Err(ArrError::Unsupported(
            "lidarr command requires a Lidarr client: use dispatch_lidarr".to_string(),
        )),
        Command::ReadarrAuthorsList
        | Command::ReadarrAuthorsLookup { .. }
        | Command::ReadarrAuthorGet { .. }
        | Command::ReadarrAuthorAdd { .. }
        | Command::ReadarrAuthorDelete { .. }
        | Command::ReadarrAuthorUpdate { .. }
        | Command::ReadarrBooksList { .. }
        | Command::ReadarrBookGet { .. }
        | Command::ReadarrBooksLookup { .. }
        | Command::ReadarrBookSearch { .. }
        | Command::ReadarrBookSetMonitored { .. }
        | Command::ReadarrBookAdd { .. }
        | Command::ReadarrBookDelete { .. }
        | Command::ReadarrBooksDeleteMany { .. }
        | Command::ReadarrAddOptions => Err(ArrError::Unsupported(
            "readarr command requires a Readarr client: use dispatch_readarr".to_string(),
        )),
        Command::BazarrWanted { .. }
        | Command::BazarrSearch { .. }
        | Command::BazarrDownload { .. }
        | Command::BazarrHistory
        | Command::BazarrProviders
        | Command::BazarrLanguages => Err(ArrError::Unsupported(
            "bazarr command requires a Bazarr client: use dispatch_bazarr".to_string(),
        )),
        Command::ProwlarrIndexersList
        | Command::ProwlarrIndexerGet { .. }
        | Command::ProwlarrIndexerAdd { .. }
        | Command::ProwlarrIndexerUpdate { .. }
        | Command::ProwlarrIndexerDelete { .. }
        | Command::ProwlarrIndexerTest { .. }
        | Command::ProwlarrIndexersTestAll
        | Command::ProwlarrIndexerSchema
        | Command::ProwlarrSearch { .. }
        | Command::ProwlarrApplicationsList
        | Command::ProwlarrApplicationGet { .. }
        | Command::ProwlarrApplicationSync { .. }
        | Command::ProwlarrApplicationsSyncAll
        | Command::ProwlarrApplicationTest { .. }
        | Command::ProwlarrHistoryList
        | Command::ProwlarrHistorySince { .. }
        | Command::ProwlarrHistoryByIndexer { .. } => Err(ArrError::Unsupported(
            "prowlarr command requires a Prowlarr client: use dispatch_prowlarr".to_string(),
        )),
        Command::OverseerrRequestsList
        | Command::OverseerrRequestGet { .. }
        | Command::OverseerrRequestCreate { .. }
        | Command::OverseerrRequestApprove { .. }
        | Command::OverseerrRequestDecline { .. }
        | Command::OverseerrRequestDelete { .. }
        | Command::OverseerrRequestsCount
        | Command::OverseerrRequestsPending
        | Command::OverseerrSearch { .. }
        | Command::OverseerrUsersList
        | Command::OverseerrUserGet { .. }
        | Command::OverseerrUserRequests { .. } => Err(ArrError::Unsupported(
            "overseerr command requires an Overseerr client: use dispatch_overseerr".to_string(),
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

/// Run one command against an explicit Lidarr client.
pub async fn dispatch_lidarr(client: &LidarrClient, command: &Command) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2d-lidarr-readarr",
            "ok": true,
        })),
        Command::LidarrArtistsList { mb_id } => client.artists_list(mb_id.as_deref()).await,
        Command::LidarrArtistsLookup { term } => client.artists_lookup(term).await,
        Command::LidarrArtistGet { artist_id } => client.artist_get(*artist_id).await,
        Command::LidarrArtistAdd {
            foreign_artist_id,
            quality_profile_id,
            root_folder_path,
            monitored,
        } => {
            client
                .artist_add(
                    foreign_artist_id,
                    *quality_profile_id,
                    root_folder_path,
                    *monitored,
                )
                .await
        }
        Command::LidarrArtistDelete {
            artist_id,
            delete_files,
        } => client.artist_delete(*artist_id, *delete_files).await,
        Command::LidarrArtistUpdate {
            artist_id,
            monitored,
        } => client.artist_update(*artist_id, *monitored).await,
        Command::LidarrAlbumsList { artist_id } => client.albums_list(*artist_id).await,
        Command::LidarrAlbumGet { album_id } => client.album_get(*album_id).await,
        Command::LidarrAlbumsLookup { term } => client.albums_lookup(term).await,
        Command::LidarrAlbumSetMonitored {
            album_id,
            monitored,
        } => client.album_set_monitored(*album_id, *monitored).await,
        _ => Err(ArrError::Unsupported(
            "non-lidarr command requires its own client".to_string(),
        )),
    }
}

/// Run one command against an explicit Readarr client.
pub async fn dispatch_readarr(
    client: &ReadarrClient,
    command: &Command,
) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2d-lidarr-readarr",
            "ok": true,
        })),
        Command::ReadarrAuthorsList => client.authors_list().await,
        Command::ReadarrAuthorsLookup { term } => client.authors_lookup(term).await,
        Command::ReadarrAuthorGet { author_id } => client.author_get(*author_id).await,
        Command::ReadarrAuthorAdd {
            foreign_author_id,
            quality_profile_id,
            root_folder_path,
            monitored,
        } => {
            client
                .author_add(
                    foreign_author_id,
                    *quality_profile_id,
                    root_folder_path,
                    *monitored,
                )
                .await
        }
        Command::ReadarrAuthorDelete {
            author_id,
            delete_files,
        } => client.author_delete(*author_id, *delete_files).await,
        Command::ReadarrAuthorUpdate {
            author_id,
            monitored,
        } => client.author_update(*author_id, *monitored).await,
        Command::ReadarrBooksList { author_id } => client.books_list(*author_id).await,
        Command::ReadarrBookGet { book_id } => client.book_get(*book_id).await,
        Command::ReadarrBooksLookup { term } => client.books_lookup(term).await,
        Command::ReadarrBookSearch { book_id } => client.book_search(*book_id).await,
        Command::ReadarrBookSetMonitored { book_id, monitored } => {
            client.book_set_monitored(*book_id, *monitored).await
        }
        Command::ReadarrBookAdd {
            foreign_book_id,
            title,
            quality_profile_id,
            root_folder_path,
            monitored,
        } => {
            client
                .book_add(
                    foreign_book_id,
                    title,
                    *quality_profile_id,
                    root_folder_path,
                    *monitored,
                )
                .await
        }
        Command::ReadarrBookDelete {
            book_id,
            delete_files,
        } => client.book_delete(*book_id, *delete_files).await,
        Command::ReadarrBooksDeleteMany {
            book_ids,
            delete_files,
        } => client.books_delete_many(book_ids, *delete_files).await,
        Command::ReadarrAddOptions => client.add_options().await,
        _ => Err(ArrError::Unsupported(
            "non-readarr command requires its own client".to_string(),
        )),
    }
}

/// Run one command against an explicit Bazarr client.
pub async fn dispatch_bazarr(client: &BazarrClient, command: &Command) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2e-bazarr-prowlarr-overseerr",
            "ok": true,
        })),
        Command::BazarrWanted { kind } => client.wanted(kind).await,
        Command::BazarrSearch {
            series_id,
            episode_id,
            movie_id,
        } => client.search(*series_id, *episode_id, *movie_id).await,
        Command::BazarrDownload { subtitle_id } => client.download(subtitle_id).await,
        Command::BazarrHistory => client.history().await,
        Command::BazarrProviders => client.providers().await,
        Command::BazarrLanguages => client.languages().await,
        _ => Err(ArrError::Unsupported(
            "non-bazarr command requires its own client".to_string(),
        )),
    }
}

/// Run one command against an explicit Prowlarr client.
pub async fn dispatch_prowlarr(
    client: &ProwlarrClient,
    command: &Command,
) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2e-bazarr-prowlarr-overseerr",
            "ok": true,
        })),
        Command::ProwlarrIndexersList => client.indexers_list().await,
        Command::ProwlarrIndexerGet { indexer_id } => client.indexer_get(*indexer_id).await,
        Command::ProwlarrIndexerAdd { fields } => client.indexer_add(fields.clone()).await,
        Command::ProwlarrIndexerUpdate { indexer_id, fields } => {
            client.indexer_update(*indexer_id, fields.clone()).await
        }
        Command::ProwlarrIndexerDelete { indexer_id } => client.indexer_delete(*indexer_id).await,
        Command::ProwlarrIndexerTest { indexer_id } => client.indexer_test(*indexer_id).await,
        Command::ProwlarrIndexersTestAll => client.indexers_test_all().await,
        Command::ProwlarrIndexerSchema => client.indexer_schema().await,
        Command::ProwlarrSearch { query } => client.search(query).await,
        Command::ProwlarrApplicationsList => client.applications_list().await,
        Command::ProwlarrApplicationGet { app_id } => client.application_get(*app_id).await,
        Command::ProwlarrApplicationSync { app_id } => client.application_sync(*app_id).await,
        Command::ProwlarrApplicationsSyncAll => client.applications_sync_all().await,
        Command::ProwlarrApplicationTest { app_id } => client.application_test(*app_id).await,
        Command::ProwlarrHistoryList => client.history_list().await,
        Command::ProwlarrHistorySince { since } => client.history_since(since).await,
        Command::ProwlarrHistoryByIndexer { indexer_id } => {
            client.history_by_indexer(*indexer_id).await
        }
        _ => Err(ArrError::Unsupported(
            "non-prowlarr command requires its own client".to_string(),
        )),
    }
}

/// Run one command against an explicit Overseerr client.
pub async fn dispatch_overseerr(
    client: &OverseerrClient,
    command: &Command,
) -> Result<Value, ArrError> {
    match command {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-2e-bazarr-prowlarr-overseerr",
            "ok": true,
        })),
        Command::OverseerrRequestsList => client.requests_list().await,
        Command::OverseerrRequestGet { request_id } => client.request_get(*request_id).await,
        Command::OverseerrRequestCreate {
            media_type,
            media_id,
            seasons,
        } => client.request_create(media_type, *media_id, seasons).await,
        Command::OverseerrRequestApprove { request_id } => {
            client.request_approve(*request_id).await
        }
        Command::OverseerrRequestDecline { request_id } => {
            client.request_decline(*request_id).await
        }
        Command::OverseerrRequestDelete { request_id } => client.request_delete(*request_id).await,
        Command::OverseerrRequestsCount => client.requests_count().await,
        Command::OverseerrRequestsPending => client.requests_pending().await,
        Command::OverseerrSearch { query } => client.search(query).await,
        Command::OverseerrUsersList => client.users_list().await,
        Command::OverseerrUserGet { user_id } => client.user_get(*user_id).await,
        Command::OverseerrUserRequests { user_id } => client.user_requests(*user_id).await,
        _ => Err(ArrError::Unsupported(
            "non-overseerr command requires its own client".to_string(),
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
        Command::LidarrArtistsList { .. }
        | Command::LidarrArtistsLookup { .. }
        | Command::LidarrArtistGet { .. }
        | Command::LidarrArtistAdd { .. }
        | Command::LidarrArtistDelete { .. }
        | Command::LidarrArtistUpdate { .. }
        | Command::LidarrAlbumsList { .. }
        | Command::LidarrAlbumGet { .. }
        | Command::LidarrAlbumsLookup { .. }
        | Command::LidarrAlbumSetMonitored { .. } => {
            let client = resolve_lidarr_client();
            dispatch_lidarr(&client, &command).await
        }
        Command::ReadarrAuthorsList
        | Command::ReadarrAuthorsLookup { .. }
        | Command::ReadarrAuthorGet { .. }
        | Command::ReadarrAuthorAdd { .. }
        | Command::ReadarrAuthorDelete { .. }
        | Command::ReadarrAuthorUpdate { .. }
        | Command::ReadarrBooksList { .. }
        | Command::ReadarrBookGet { .. }
        | Command::ReadarrBooksLookup { .. }
        | Command::ReadarrBookSearch { .. }
        | Command::ReadarrBookSetMonitored { .. }
        | Command::ReadarrBookAdd { .. }
        | Command::ReadarrBookDelete { .. }
        | Command::ReadarrBooksDeleteMany { .. }
        | Command::ReadarrAddOptions => {
            let client = resolve_readarr_client();
            dispatch_readarr(&client, &command).await
        }
        Command::BazarrWanted { .. }
        | Command::BazarrSearch { .. }
        | Command::BazarrDownload { .. }
        | Command::BazarrHistory
        | Command::BazarrProviders
        | Command::BazarrLanguages => {
            let client = resolve_bazarr_client();
            dispatch_bazarr(&client, &command).await
        }
        Command::ProwlarrIndexersList
        | Command::ProwlarrIndexerGet { .. }
        | Command::ProwlarrIndexerAdd { .. }
        | Command::ProwlarrIndexerUpdate { .. }
        | Command::ProwlarrIndexerDelete { .. }
        | Command::ProwlarrIndexerTest { .. }
        | Command::ProwlarrIndexersTestAll
        | Command::ProwlarrIndexerSchema
        | Command::ProwlarrSearch { .. }
        | Command::ProwlarrApplicationsList
        | Command::ProwlarrApplicationGet { .. }
        | Command::ProwlarrApplicationSync { .. }
        | Command::ProwlarrApplicationsSyncAll
        | Command::ProwlarrApplicationTest { .. }
        | Command::ProwlarrHistoryList
        | Command::ProwlarrHistorySince { .. }
        | Command::ProwlarrHistoryByIndexer { .. } => {
            let client = resolve_prowlarr_client();
            dispatch_prowlarr(&client, &command).await
        }
        Command::OverseerrRequestsList
        | Command::OverseerrRequestGet { .. }
        | Command::OverseerrRequestCreate { .. }
        | Command::OverseerrRequestApprove { .. }
        | Command::OverseerrRequestDecline { .. }
        | Command::OverseerrRequestDelete { .. }
        | Command::OverseerrRequestsCount
        | Command::OverseerrRequestsPending
        | Command::OverseerrSearch { .. }
        | Command::OverseerrUsersList
        | Command::OverseerrUserGet { .. }
        | Command::OverseerrUserRequests { .. } => {
            let client = resolve_overseerr_client();
            dispatch_overseerr(&client, &command).await
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

fn resolve_service_config(service: &str) -> Option<(String, String)> {
    // 1. Explicit process env.
    let prefix = service.to_ascii_uppercase();
    let env_url = std::env::var(format!("{prefix}_URL"))
        .ok()
        .filter(|v| !v.is_empty());
    let env_key = std::env::var(format!("{prefix}_API_KEY"))
        .ok()
        .filter(|v| !v.is_empty());
    if let (Some(url), Some(key)) = (env_url, env_key) {
        return Some((url, key));
    }
    // 2. Imported legacy checkout.
    if let Some(dir) = discover_legacy_dir() {
        if let Ok(services) = load_from_dir(&dir) {
            if let Some(cfg) = services.get(service) {
                if let (Some(url), Some(key)) = (cfg.url.clone(), cfg.api_key.clone()) {
                    return Some((url, key));
                }
            }
        }
    }
    None
}

fn resolve_lidarr_client() -> LidarrClient {
    if let Some((url, key)) = resolve_service_config("lidarr") {
        if let Ok(client) = LidarrClient::new(&url, &key) {
            return client;
        }
    }
    LidarrClient::demo()
}

fn resolve_readarr_client() -> ReadarrClient {
    if let Some((url, key)) = resolve_service_config("readarr") {
        if let Ok(client) = ReadarrClient::new(&url, &key) {
            return client;
        }
    }
    ReadarrClient::demo()
}

fn resolve_bazarr_client() -> BazarrClient {
    if let Some((url, key)) = resolve_service_config("bazarr") {
        if let Ok(client) = BazarrClient::new(&url, &key) {
            return client;
        }
    }
    BazarrClient::demo()
}

fn resolve_prowlarr_client() -> ProwlarrClient {
    if let Some((url, key)) = resolve_service_config("prowlarr") {
        if let Ok(client) = ProwlarrClient::new(&url, &key) {
            return client;
        }
    }
    ProwlarrClient::demo()
}

fn resolve_overseerr_client() -> OverseerrClient {
    if let Some((url, key)) = resolve_service_config("overseerr") {
        if let Ok(client) = OverseerrClient::new(&url, &key) {
            return client;
        }
    }
    OverseerrClient::demo()
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

    #[tokio::test]
    async fn demo_lidarr_explicit() {
        use crate::lidarr::LidarrClient;
        let client = LidarrClient::demo();
        let v = dispatch_lidarr(&client, &Command::LidarrArtistsList { mb_id: None })
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }

    #[tokio::test]
    async fn demo_readarr_explicit() {
        use crate::readarr::ReadarrClient;
        let client = ReadarrClient::demo();
        let v = dispatch_readarr(&client, &Command::ReadarrAuthorsList)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }

    #[tokio::test]
    async fn demo_bazarr_explicit() {
        use crate::bazarr::BazarrClient;
        let client = BazarrClient::demo();
        let v = dispatch_bazarr(&client, &Command::BazarrProviders)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }

    #[tokio::test]
    async fn demo_prowlarr_explicit() {
        use crate::prowlarr::ProwlarrClient;
        let client = ProwlarrClient::demo();
        let v = dispatch_prowlarr(&client, &Command::ProwlarrIndexersList)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }

    #[tokio::test]
    async fn demo_overseerr_explicit() {
        use crate::overseerr::OverseerrClient;
        let client = OverseerrClient::demo();
        let v = dispatch_overseerr(&client, &Command::OverseerrRequestsList)
            .await
            .unwrap_or(Value::Null);
        assert_eq!(v.as_array().map(Vec::len), Some(2));
    }
}
