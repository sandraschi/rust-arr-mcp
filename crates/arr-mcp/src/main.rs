//! L2 `rust-arr-mcp`: CLI-first binary + MCP `2024-11-05` stdio server over arr-core.
//!
//! Stdout is pure JSON-RPC. Every log line goes to stderr via `eprintln!`.
//! All operations route through `arr_core::command`. Request IDs echo verbatim
//! (numeric or string) per the fleet `id_to_key` lesson.
//!
//! CLI shape (documented deviation): per-service subcommands with a typed `--op`
//! enum + `--params-json`, rather than 80 one-shot subcommands. Every subcommand
//! honors `--json`. Rationale in `docs/CONVERSION_LOG.md` (Phase 3 entry).

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use arr_core::bazarr::BazarrClient;
use arr_core::command::{dispatch_auto, Command};
use arr_core::error::ArrError;
use arr_core::lidarr::LidarrClient;
use arr_core::overseerr::OverseerrClient;
use arr_core::prowlarr::ProwlarrClient;
use arr_core::radarr::RadarrClient;
use arr_core::readarr::ReadarrClient;
use arr_core::sonarr::SonarrClient;
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};

fn install_forensic_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let msg = format!("[{ts}] PANIC in rust-arr-mcp: {info}\n");
        let _ = io::stderr().write_all(msg.as_bytes());
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let dir = PathBuf::from(local).join("rust-arr-mcp");
            let _ = std::fs::create_dir_all(&dir);
            let log_file = dir.join("crash.log");
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
            {
                let _ = f.write_all(msg.as_bytes());
            }
        }
    }));
}

#[derive(Parser)]
#[command(
    name = "rust-arr-mcp",
    version,
    about = "Arr stack CLI + MCP 2024-11-05 Server",
    disable_help_subcommand = true
)]
struct Cli {
    /// Emit compact JSON on CLI commands
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Service status (no network).
    Status,
    /// Sonarr TV operations.
    Sonarr {
        #[arg(value_enum)]
        op: SonarrOp,
        /// JSON object with op params.
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long, global = false)]
        base_url: Option<String>,
        #[arg(long, global = false)]
        api_key: Option<String>,
    },
    /// Radarr movie operations.
    Radarr {
        #[arg(value_enum)]
        op: RadarrOp,
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Lidarr music operations.
    Lidarr {
        #[arg(value_enum)]
        op: LidarrOp,
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Readarr book operations.
    Readarr {
        #[arg(value_enum)]
        op: ReadarrOp,
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Bazarr subtitle operations.
    Bazarr {
        #[arg(value_enum)]
        op: BazarrOp,
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Prowlarr indexer operations.
    Prowlarr {
        #[arg(value_enum)]
        op: ProwlarrOp,
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Overseerr request operations.
    Overseerr {
        #[arg(value_enum)]
        op: OverseerrOp,
        #[arg(long, default_value = "{}")]
        params_json: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Stack health (config presence per service; --probe does one live read each).
    Health {
        #[arg(long, default_value_t = false)]
        probe: bool,
    },
    /// Help: service inventory + tool counts.
    Help,
    /// Run MCP 2024-11-05 JSON-RPC stdio server loop.
    #[command(name = "mcp")]
    Mcp,
}

#[derive(Copy, Clone, ValueEnum)]
enum SonarrOp {
    SeriesList,
    SeriesLookup,
    SeriesGet,
    SeriesAdd,
    SeriesSearch,
    SeriesDelete,
    SeriesUpdate,
    EpisodesList,
    EpisodeGet,
    EpisodeSearch,
    EpisodeSetMonitored,
}

#[derive(Copy, Clone, ValueEnum)]
enum RadarrOp {
    MoviesList,
    MoviesLookup,
    MovieGet,
    MovieAdd,
    MovieSearch,
    MovieDelete,
    MovieUpdate,
    MoviesImport,
}

#[derive(Copy, Clone, ValueEnum)]
enum LidarrOp {
    ArtistsList,
    ArtistsLookup,
    ArtistGet,
    ArtistAdd,
    ArtistDelete,
    ArtistUpdate,
    AlbumsList,
    AlbumGet,
    AlbumsLookup,
    AlbumSetMonitored,
}

#[derive(Copy, Clone, ValueEnum)]
enum ReadarrOp {
    AuthorsList,
    AuthorsLookup,
    AuthorGet,
    AuthorAdd,
    AuthorDelete,
    AuthorUpdate,
    BooksList,
    BookGet,
    BooksLookup,
    BookSearch,
    BookSetMonitored,
    BookAdd,
    BookDelete,
    BooksDeleteMany,
    AddOptions,
}

#[derive(Copy, Clone, ValueEnum)]
enum BazarrOp {
    Wanted,
    Search,
    Download,
    History,
    Providers,
    Languages,
}

#[derive(Copy, Clone, ValueEnum)]
enum ProwlarrOp {
    IndexersList,
    IndexerGet,
    IndexerAdd,
    IndexerUpdate,
    IndexerDelete,
    IndexerTest,
    IndexersTestAll,
    IndexerSchema,
    Search,
    ApplicationsList,
    ApplicationGet,
    ApplicationSync,
    ApplicationsSyncAll,
    ApplicationTest,
    HistoryList,
    HistorySince,
    HistoryByIndexer,
}

#[derive(Copy, Clone, ValueEnum)]
enum OverseerrOp {
    RequestsList,
    RequestGet,
    RequestCreate,
    RequestApprove,
    RequestDecline,
    RequestDelete,
    RequestsCount,
    RequestsPending,
    Search,
    UsersList,
    UserGet,
    UserRequests,
}

// --- param extraction (typed, no silent defaults on required values) ---

fn req_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("{key} is required"))
}

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

fn req_i64(args: &Value, key: &str) -> Result<i64, String> {
    args.get(key)
        .and_then(Value::as_i64)
        .filter(|v| *v != 0 || key.ends_with("number"))
        .ok_or_else(|| format!("{key} is required"))
}

fn opt_i64(args: &Value, key: &str) -> Option<i64> {
    args.get(key).and_then(Value::as_i64)
}

fn opt_bool(args: &Value, key: &str, default: bool) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn opt_i64_list(args: &Value, key: &str) -> Vec<i64> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default()
}

fn req_obj(args: &Value, key: &str) -> Result<Value, String> {
    args.get(key)
        .filter(|v| v.is_object())
        .cloned()
        .ok_or_else(|| format!("{key} must be an object"))
}

// --- service/op/name → Command (shared by CLI and MCP) ---

fn to_command(service: &str, op: &str, args: &Value) -> Result<Command, String> {
    match (service, op) {
        ("sonarr", "series_list") => Ok(Command::SonarrSeriesList),
        ("sonarr", "series_lookup") => Ok(Command::SonarrSeriesLookup {
            term: req_str(args, "term")?,
        }),
        ("sonarr", "series_get") => Ok(Command::SonarrSeriesGet {
            series_id: req_i64(args, "series_id")?,
        }),
        ("sonarr", "series_add") => Ok(Command::SonarrSeriesAdd {
            tvdb_id: req_i64(args, "tvdb_id")?,
            title: opt_str(args, "title").unwrap_or_default(),
            quality_profile_id: req_i64(args, "quality_profile_id")?,
            root_folder_path: req_str(args, "root_folder_path")?,
            monitored: opt_bool(args, "monitored", true),
            search_for_missing: opt_bool(args, "search_for_missing", true),
        }),
        ("sonarr", "series_search") => Ok(Command::SonarrSeriesSearch {
            series_id: req_i64(args, "series_id")?,
        }),
        ("sonarr", "series_delete") => Ok(Command::SonarrSeriesDelete {
            series_id: req_i64(args, "series_id")?,
            delete_files: opt_bool(args, "delete_files", false),
            add_exclusion: opt_bool(args, "add_import_list_exclusion", false),
        }),
        ("sonarr", "series_update") => Ok(Command::SonarrSeriesUpdate {
            series_id: req_i64(args, "series_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("sonarr", "episodes_list") => Ok(Command::SonarrEpisodesList {
            series_id: req_i64(args, "series_id")?,
            season_number: opt_i64(args, "season_number"),
        }),
        ("sonarr", "episode_get") => Ok(Command::SonarrEpisodeGet {
            episode_id: req_i64(args, "episode_id")?,
        }),
        ("sonarr", "episode_search") => Ok(Command::SonarrEpisodeSearch {
            episode_id: opt_i64(args, "episode_id"),
            series_id: opt_i64(args, "series_id"),
            season_number: opt_i64(args, "season_number"),
        }),
        ("sonarr", "episode_set_monitored") => Ok(Command::SonarrEpisodeSetMonitored {
            episode_id: req_i64(args, "episode_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("radarr", "movies_list") => Ok(Command::RadarrMoviesList),
        ("radarr", "movies_lookup") => Ok(Command::RadarrMoviesLookup {
            term: req_str(args, "term")?,
        }),
        ("radarr", "movie_get") => Ok(Command::RadarrMovieGet {
            movie_id: req_i64(args, "movie_id")?,
        }),
        ("radarr", "movie_add") => Ok(Command::RadarrMovieAdd {
            tmdb_id: req_i64(args, "tmdb_id")?,
            title: opt_str(args, "title").unwrap_or_default(),
            quality_profile_id: req_i64(args, "quality_profile_id")?,
            root_folder_path: req_str(args, "root_folder_path")?,
            monitored: opt_bool(args, "monitored", true),
            search_for_movie: opt_bool(args, "search_for_movie", true),
        }),
        ("radarr", "movie_search") => Ok(Command::RadarrMovieSearch {
            movie_id: req_i64(args, "movie_id")?,
        }),
        ("radarr", "movie_delete") => Ok(Command::RadarrMovieDelete {
            movie_id: req_i64(args, "movie_id")?,
            delete_files: opt_bool(args, "delete_files", false),
            add_exclusion: opt_bool(args, "add_import_list_exclusion", false),
        }),
        ("radarr", "movie_update") => Ok(Command::RadarrMovieUpdate {
            movie_id: req_i64(args, "movie_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("radarr", "movies_import") => Ok(Command::RadarrMoviesImport {
            folder: opt_str(args, "folder"),
        }),
        ("lidarr", "artists_list") => Ok(Command::LidarrArtistsList {
            mb_id: opt_str(args, "mb_id"),
        }),
        ("lidarr", "artists_lookup") => Ok(Command::LidarrArtistsLookup {
            term: req_str(args, "term")?,
        }),
        ("lidarr", "artist_get") => Ok(Command::LidarrArtistGet {
            artist_id: req_i64(args, "artist_id")?,
        }),
        ("lidarr", "artist_add") => Ok(Command::LidarrArtistAdd {
            foreign_artist_id: req_str(args, "foreign_artist_id")?,
            quality_profile_id: req_i64(args, "quality_profile_id")?,
            root_folder_path: req_str(args, "root_folder_path")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("lidarr", "artist_delete") => Ok(Command::LidarrArtistDelete {
            artist_id: req_i64(args, "artist_id")?,
            delete_files: opt_bool(args, "delete_files", false),
        }),
        ("lidarr", "artist_update") => Ok(Command::LidarrArtistUpdate {
            artist_id: req_i64(args, "artist_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("lidarr", "albums_list") => Ok(Command::LidarrAlbumsList {
            artist_id: opt_i64(args, "artist_id"),
        }),
        ("lidarr", "album_get") => Ok(Command::LidarrAlbumGet {
            album_id: req_i64(args, "album_id")?,
        }),
        ("lidarr", "albums_lookup") => Ok(Command::LidarrAlbumsLookup {
            term: req_str(args, "term")?,
        }),
        ("lidarr", "album_set_monitored") => Ok(Command::LidarrAlbumSetMonitored {
            album_id: req_i64(args, "album_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("readarr", "authors_list") => Ok(Command::ReadarrAuthorsList),
        ("readarr", "authors_lookup") => Ok(Command::ReadarrAuthorsLookup {
            term: req_str(args, "term")?,
        }),
        ("readarr", "author_get") => Ok(Command::ReadarrAuthorGet {
            author_id: req_i64(args, "author_id")?,
        }),
        ("readarr", "author_add") => Ok(Command::ReadarrAuthorAdd {
            foreign_author_id: req_str(args, "foreign_author_id")?,
            quality_profile_id: req_i64(args, "quality_profile_id")?,
            root_folder_path: req_str(args, "root_folder_path")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("readarr", "author_delete") => Ok(Command::ReadarrAuthorDelete {
            author_id: req_i64(args, "author_id")?,
            delete_files: opt_bool(args, "delete_files", false),
        }),
        ("readarr", "author_update") => Ok(Command::ReadarrAuthorUpdate {
            author_id: req_i64(args, "author_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("readarr", "books_list") => Ok(Command::ReadarrBooksList {
            author_id: opt_i64(args, "author_id"),
        }),
        ("readarr", "book_get") => Ok(Command::ReadarrBookGet {
            book_id: req_i64(args, "book_id")?,
        }),
        ("readarr", "books_lookup") => Ok(Command::ReadarrBooksLookup {
            term: req_str(args, "term")?,
        }),
        ("readarr", "book_search") => Ok(Command::ReadarrBookSearch {
            book_id: req_i64(args, "book_id")?,
        }),
        ("readarr", "book_set_monitored") => Ok(Command::ReadarrBookSetMonitored {
            book_id: req_i64(args, "book_id")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("readarr", "book_add") => Ok(Command::ReadarrBookAdd {
            foreign_book_id: req_str(args, "foreign_book_id")?,
            title: opt_str(args, "title").unwrap_or_default(),
            quality_profile_id: req_i64(args, "quality_profile_id")?,
            root_folder_path: req_str(args, "root_folder_path")?,
            monitored: opt_bool(args, "monitored", true),
        }),
        ("readarr", "book_delete") => Ok(Command::ReadarrBookDelete {
            book_id: req_i64(args, "book_id")?,
            delete_files: opt_bool(args, "delete_files", false),
        }),
        ("readarr", "books_delete_many") => {
            let ids = args
                .get("book_ids")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| "book_ids must be a non-empty array".to_string())?;
            Ok(Command::ReadarrBooksDeleteMany {
                book_ids: ids,
                delete_files: opt_bool(args, "delete_files", false),
            })
        }
        ("readarr", "add_options") => Ok(Command::ReadarrAddOptions),
        ("bazarr", "wanted") => Ok(Command::BazarrWanted {
            kind: opt_str(args, "kind").unwrap_or_else(|| "all".to_string()),
        }),
        ("bazarr", "search") => Ok(Command::BazarrSearch {
            series_id: opt_i64(args, "series_id"),
            episode_id: opt_i64(args, "episode_id"),
            movie_id: opt_i64(args, "movie_id"),
        }),
        ("bazarr", "download") => Ok(Command::BazarrDownload {
            subtitle_id: req_str(args, "subtitle_id")?,
        }),
        ("bazarr", "history") => Ok(Command::BazarrHistory),
        ("bazarr", "providers") => Ok(Command::BazarrProviders),
        ("bazarr", "languages") => Ok(Command::BazarrLanguages),
        ("prowlarr", "indexers_list") => Ok(Command::ProwlarrIndexersList),
        ("prowlarr", "indexer_get") => Ok(Command::ProwlarrIndexerGet {
            indexer_id: req_i64(args, "indexer_id")?,
        }),
        ("prowlarr", "indexer_add") => Ok(Command::ProwlarrIndexerAdd {
            fields: req_obj(args, "fields")?,
        }),
        ("prowlarr", "indexer_update") => Ok(Command::ProwlarrIndexerUpdate {
            indexer_id: req_i64(args, "indexer_id")?,
            fields: req_obj(args, "fields")?,
        }),
        ("prowlarr", "indexer_delete") => Ok(Command::ProwlarrIndexerDelete {
            indexer_id: req_i64(args, "indexer_id")?,
        }),
        ("prowlarr", "indexer_test") => Ok(Command::ProwlarrIndexerTest {
            indexer_id: req_i64(args, "indexer_id")?,
        }),
        ("prowlarr", "indexers_test_all") => Ok(Command::ProwlarrIndexersTestAll),
        ("prowlarr", "indexer_schema") => Ok(Command::ProwlarrIndexerSchema),
        ("prowlarr", "search") => Ok(Command::ProwlarrSearch {
            query: req_str(args, "query")?,
        }),
        ("prowlarr", "applications_list") => Ok(Command::ProwlarrApplicationsList),
        ("prowlarr", "application_get") => Ok(Command::ProwlarrApplicationGet {
            app_id: req_i64(args, "app_id")?,
        }),
        ("prowlarr", "application_sync") => Ok(Command::ProwlarrApplicationSync {
            app_id: req_i64(args, "app_id")?,
        }),
        ("prowlarr", "applications_sync_all") => Ok(Command::ProwlarrApplicationsSyncAll),
        ("prowlarr", "application_test") => Ok(Command::ProwlarrApplicationTest {
            app_id: req_i64(args, "app_id")?,
        }),
        ("prowlarr", "history_list") => Ok(Command::ProwlarrHistoryList),
        ("prowlarr", "history_since") => Ok(Command::ProwlarrHistorySince {
            since: req_str(args, "since")?,
        }),
        ("prowlarr", "history_by_indexer") => Ok(Command::ProwlarrHistoryByIndexer {
            indexer_id: req_i64(args, "indexer_id")?,
        }),
        ("overseerr", "requests_list") => Ok(Command::OverseerrRequestsList),
        ("overseerr", "request_get") => Ok(Command::OverseerrRequestGet {
            request_id: req_i64(args, "request_id")?,
        }),
        ("overseerr", "request_create") => Ok(Command::OverseerrRequestCreate {
            media_type: req_str(args, "media_type")?,
            media_id: req_i64(args, "media_id")?,
            seasons: opt_i64_list(args, "seasons"),
        }),
        ("overseerr", "request_approve") => Ok(Command::OverseerrRequestApprove {
            request_id: req_i64(args, "request_id")?,
        }),
        ("overseerr", "request_decline") => Ok(Command::OverseerrRequestDecline {
            request_id: req_i64(args, "request_id")?,
        }),
        ("overseerr", "request_delete") => Ok(Command::OverseerrRequestDelete {
            request_id: req_i64(args, "request_id")?,
        }),
        ("overseerr", "requests_count") => Ok(Command::OverseerrRequestsCount),
        ("overseerr", "requests_pending") => Ok(Command::OverseerrRequestsPending),
        ("overseerr", "search") => Ok(Command::OverseerrSearch {
            query: req_str(args, "query")?,
        }),
        ("overseerr", "users_list") => Ok(Command::OverseerrUsersList),
        ("overseerr", "user_get") => Ok(Command::OverseerrUserGet {
            user_id: req_i64(args, "user_id")?,
        }),
        ("overseerr", "user_requests") => Ok(Command::OverseerrUserRequests {
            user_id: req_i64(args, "user_id")?,
        }),
        (service, op) => Err(format!("unknown service/op '{service}/{op}'")),
    }
}

/// Split a tool name into (service, op). Accepts `sonarr.series_list`,
/// `sonarr_series_list`, and legacy `sonarr_series` + `operation` arg.
fn split_tool_name(name: &str, args: &Value) -> Result<(String, String), String> {
    if let Some(dot) = name.find('.') {
        let (service, op) = name.split_at(dot);
        return Ok((service.to_string(), op[1..].to_string()));
    }
    // Legacy portmanteau: `sonarr_series` + operation="list" → ("sonarr","series_list").
    // Explicit table (no string surgery that could misroute).
    let legacy: &[(&str, &str)] = &[
        ("sonarr_series", "sonarr"),
        ("sonarr_episodes", "sonarr"),
        ("radarr_movies", "radarr"),
        ("lidarr_artists", "lidarr"),
        ("lidarr_albums", "lidarr"),
        ("readarr_authors", "readarr"),
        ("readarr_books", "readarr"),
        ("readarr_add_book", "readarr"),
        ("readarr_delete_book", "readarr"),
        ("readarr_delete_books", "readarr"),
        ("readarr_add_options", "readarr"),
        ("bazarr_subtitles", "bazarr"),
        ("prowlarr_indexers", "prowlarr"),
        ("prowlarr_search", "prowlarr"),
        ("prowlarr_applications", "prowlarr"),
        ("prowlarr_history", "prowlarr"),
        ("overseerr_requests", "overseerr"),
        ("overseerr_search", "overseerr"),
        ("overseerr_users", "overseerr"),
        ("arr_orchestrate", "arr"),
        ("arr_calendar", "arr"),
        ("arr_stats", "arr"),
        ("arr_health", "arr"),
        ("arr_help", "arr"),
        ("arr_agentic", "arr"),
    ];
    if let Some((_, service)) = legacy.iter().find(|(tool, _)| *tool == name) {
        let op = args
            .get("operation")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| format!("legacy tool '{name}' requires an operation argument"))?;
        // Map legacy (tool, operation) → (service, op-variant).
        let mapped = match (name, op) {
            ("sonarr_series", "list") => "series_list",
            ("sonarr_series", "lookup") => "series_lookup",
            ("sonarr_series", "get") => "series_get",
            ("sonarr_series", "add") => "series_add",
            ("sonarr_series", "search") => "series_search",
            ("sonarr_series", "delete") => "series_delete",
            ("sonarr_series", "update") => "series_update",
            ("sonarr_episodes", "list") => "episodes_list",
            ("sonarr_episodes", "get") => "episode_get",
            ("sonarr_episodes", "search") => "episode_search",
            ("sonarr_episodes", "set_monitored") => "episode_set_monitored",
            ("radarr_movies", "list") => "movies_list",
            ("radarr_movies", "lookup") => "movies_lookup",
            ("radarr_movies", "get") => "movie_get",
            ("radarr_movies", "add") => "movie_add",
            ("radarr_movies", "search") => "movie_search",
            ("radarr_movies", "delete") => "movie_delete",
            ("radarr_movies", "update") => "movie_update",
            ("radarr_movies", "import") => "movies_import",
            ("lidarr_artists", o) => match o {
                "list" => "artists_list",
                "lookup" => "artists_lookup",
                "get" => "artist_get",
                "add" => "artist_add",
                "delete" => "artist_delete",
                "update" => "artist_update",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("lidarr_albums", o) => match o {
                "list" => "albums_list",
                "get" => "album_get",
                "lookup" => "albums_lookup",
                "set_monitored" => "album_set_monitored",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("readarr_authors", o) => match o {
                "list" => "authors_list",
                "lookup" => "authors_lookup",
                "get" => "author_get",
                "add" => "author_add",
                "delete" => "author_delete",
                "update" => "author_update",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("readarr_books", o) => match o {
                "list" => "books_list",
                "get" => "book_get",
                "lookup" => "books_lookup",
                "search" => "book_search",
                "set_monitored" => "book_set_monitored",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("readarr_add_book", _) => "book_add",
            ("readarr_delete_book", _) => "book_delete",
            ("readarr_delete_books", _) => "books_delete_many",
            ("readarr_add_options", _) => "add_options",
            ("bazarr_subtitles", o) => match o {
                "wanted" => "wanted",
                "search" => "search",
                "download" => "download",
                "history" => "history",
                "providers" => "providers",
                "languages" => "languages",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("prowlarr_indexers", o) => match o {
                "list" => "indexers_list",
                "get" => "indexer_get",
                "add" => "indexer_add",
                "update" => "indexer_update",
                "delete" => "indexer_delete",
                "test" => "indexer_test",
                "test_all" => "indexers_test_all",
                "schema" => "indexer_schema",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("prowlarr_search", _) => "search",
            ("prowlarr_applications", o) => match o {
                "list" => "applications_list",
                "get" => "application_get",
                "sync" => "application_sync",
                "sync_all" => "applications_sync_all",
                "test" => "application_test",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("prowlarr_history", o) => match o {
                "list" => "history_list",
                "since" => "history_since",
                "by_indexer" => "history_by_indexer",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("overseerr_requests", o) => match o {
                "list" => "requests_list",
                "get" => "request_get",
                "create" => "request_create",
                "approve" => "request_approve",
                "decline" => "request_decline",
                "delete" => "request_delete",
                "count" => "requests_count",
                "pending" => "requests_pending",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("overseerr_search", _) => "search",
            ("overseerr_users", o) => match o {
                "list" => "users_list",
                "get" => "user_get",
                "requests" => "user_requests",
                _ => return Err(format!("unknown operation '{op}' for '{name}'")),
            },
            ("arr_orchestrate", _)
            | ("arr_calendar", _)
            | ("arr_stats", _)
            | ("arr_health", _)
            | ("arr_help", _)
            | ("arr_agentic", _) => {
                return Err(format!(
                "legacy tool '{name}' is orchestration-layer (Phase 3 compose/L3): not yet ported"
            ))
            }
            _ => return Err(format!("unknown operation '{op}' for '{name}'")),
        };
        return Ok((service.to_string(), mapped.to_string()));
    }
    // Underscore-joined form: `sonarr_series_list` → ("sonarr", "series_list").
    // Service prefix must match a known service (no blind splitting).
    for service in [
        "sonarr",
        "radarr",
        "lidarr",
        "readarr",
        "bazarr",
        "prowlarr",
        "overseerr",
        "arr",
    ] {
        if let Some(rest) = name.strip_prefix(&format!("{service}_")) {
            if rest.contains('_') || !rest.is_empty() {
                return Ok((service.to_string(), rest.to_string()));
            }
        }
    }
    Err(format!("unknown tool '{name}'"))
}

async fn call_tool(name: &str, args: Value) -> Result<Value, ArrError> {
    if name == "arr.status" || name == "arr_status" {
        return dispatch_auto(Command::Status).await;
    }
    if name == "arr.health" || name == "arr_health" {
        return Ok(health_snapshot().await);
    }
    if name == "arr.help" || name == "arr_help" {
        return Ok(help_snapshot());
    }
    let (service, op) = split_tool_name(name, &args).map_err(ArrError::InvalidInput)?;
    let command = to_command(&service, &op, &args).map_err(ArrError::InvalidInput)?;
    dispatch_auto(command).await
}

async fn health_snapshot() -> Value {
    // Config presence per service (no network on the stdio path by default).
    // Live probing stays behind the CLI `--probe` flag.
    let present = |service: &str| {
        let prefix = service.to_ascii_uppercase();
        let has_env = std::env::var(format!("{prefix}_URL")).is_ok()
            && std::env::var(format!("{prefix}_API_KEY")).is_ok();
        has_env
    };
    let mut services = serde_json::Map::new();
    for service in [
        "sonarr",
        "radarr",
        "lidarr",
        "readarr",
        "bazarr",
        "prowlarr",
        "overseerr",
    ] {
        services.insert(
            service.to_string(),
            json!({"configured_env": present(service), "mode": "demo-or-live-via-dispatch_auto"}),
        );
    }
    json!({"service": "rust-arr-mcp", "ok": true, "services": services})
}

fn help_snapshot() -> Value {
    json!({
        "service": "rust-arr-mcp",
        "services": ["sonarr", "radarr", "lidarr", "readarr", "bazarr", "prowlarr", "overseerr"],
        "tool_count": TOOL_CATALOG.len(),
        "naming": ["<service>.<action>", "<service>_<action>", "legacy <tool>(operation=...)"],
        "deferred": ["arr_orchestrate", "arr_calendar", "arr_stats", "arr_agentic", "prefab cards"],
    })
}

// --- MCP 2024-11-05 Tool Catalog (80 tools) ---

const TOOL_CATALOG: &[(&str, &str, &str)] = &[
    ("arr.status", "Show service status (no network).", r#"{"type":"object","properties":{}}"#),
    ("arr.health", "Stack health per service (config presence, no network).", r#"{"type":"object","properties":{}}"#),
    ("arr.help", "Service inventory and tool counts.", r#"{"type":"object","properties":{}}"#),
    ("sonarr.series_list", "List all Sonarr series.", r#"{"type":"object","properties":{}}"#),
    ("sonarr.series_lookup", "Look up series metadata by term.", r#"{"type":"object","properties":{"term":{"type":"string"}},"required":["term"]}"#),
    ("sonarr.series_get", "Get one series by Sonarr ID.", r#"{"type":"object","properties":{"series_id":{"type":"integer"}},"required":["series_id"]}"#),
    ("sonarr.series_add", "Add a series by TVDB ID. Requires quality_profile_id and root_folder_path.", r#"{"type":"object","properties":{"tvdb_id":{"type":"integer"},"title":{"type":"string"},"quality_profile_id":{"type":"integer"},"root_folder_path":{"type":"string"},"monitored":{"type":"boolean"},"search_for_missing":{"type":"boolean"}},"required":["tvdb_id","quality_profile_id","root_folder_path"]}"#),
    ("sonarr.series_search", "Trigger SeriesSearch for a series already in the library (confirms submission).", r#"{"type":"object","properties":{"series_id":{"type":"integer"}},"required":["series_id"]}"#),
    ("sonarr.series_delete", "Delete a series. Set delete_files only to also remove files.", r#"{"type":"object","properties":{"series_id":{"type":"integer"},"delete_files":{"type":"boolean"},"add_import_list_exclusion":{"type":"boolean"}},"required":["series_id"]}"#),
    ("sonarr.series_update", "Update a series monitored flag.", r#"{"type":"object","properties":{"series_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["series_id"]}"#),
    ("sonarr.episodes_list", "List episodes for a series, optional season filter.", r#"{"type":"object","properties":{"series_id":{"type":"integer"},"season_number":{"type":"integer"}},"required":["series_id"]}"#),
    ("sonarr.episode_get", "Get one episode by episode ID.", r#"{"type":"object","properties":{"episode_id":{"type":"integer"}},"required":["episode_id"]}"#),
    ("sonarr.episode_search", "Search one episode (episode_id) or a whole season (series_id + season_number).", r#"{"type":"object","properties":{"episode_id":{"type":"integer"},"series_id":{"type":"integer"},"season_number":{"type":"integer"}}}"#),
    ("sonarr.episode_set_monitored", "Set an episode monitored flag.", r#"{"type":"object","properties":{"episode_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["episode_id"]}"#),
    ("radarr.movies_list", "List all Radarr movies.", r#"{"type":"object","properties":{}}"#),
    ("radarr.movies_lookup", "Look up movie metadata by term.", r#"{"type":"object","properties":{"term":{"type":"string"}},"required":["term"]}"#),
    ("radarr.movie_get", "Get one movie by Radarr ID.", r#"{"type":"object","properties":{"movie_id":{"type":"integer"}},"required":["movie_id"]}"#),
    ("radarr.movie_add", "Add a movie by TMDB ID. Requires quality_profile_id and root_folder_path.", r#"{"type":"object","properties":{"tmdb_id":{"type":"integer"},"title":{"type":"string"},"quality_profile_id":{"type":"integer"},"root_folder_path":{"type":"string"},"monitored":{"type":"boolean"},"search_for_movie":{"type":"boolean"}},"required":["tmdb_id","quality_profile_id","root_folder_path"]}"#),
    ("radarr.movie_search", "Trigger MoviesSearch for a movie already in the library.", r#"{"type":"object","properties":{"movie_id":{"type":"integer"}},"required":["movie_id"]}"#),
    ("radarr.movie_delete", "Delete a movie. Set delete_files only to also remove files.", r#"{"type":"object","properties":{"movie_id":{"type":"integer"},"delete_files":{"type":"boolean"},"add_import_list_exclusion":{"type":"boolean"}},"required":["movie_id"]}"#),
    ("radarr.movie_update", "Update a movie monitored flag.", r#"{"type":"object","properties":{"movie_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["movie_id"]}"#),
    ("radarr.movies_import", "List manual import candidates, optional folder.", r#"{"type":"object","properties":{"folder":{"type":"string"}}}"#),
    ("lidarr.artists_list", "List Lidarr artists, optional MusicBrainz ID filter.", r#"{"type":"object","properties":{"mb_id":{"type":"string"}}}"#),
    ("lidarr.artists_lookup", "Look up artist metadata by term.", r#"{"type":"object","properties":{"term":{"type":"string"}},"required":["term"]}"#),
    ("lidarr.artist_get", "Get one artist by Lidarr ID.", r#"{"type":"object","properties":{"artist_id":{"type":"integer"}},"required":["artist_id"]}"#),
    ("lidarr.artist_add", "Add an artist by MusicBrainz foreign ID. Requires quality_profile_id and root_folder_path.", r#"{"type":"object","properties":{"foreign_artist_id":{"type":"string"},"quality_profile_id":{"type":"integer"},"root_folder_path":{"type":"string"},"monitored":{"type":"boolean"}},"required":["foreign_artist_id","quality_profile_id","root_folder_path"]}"#),
    ("lidarr.artist_delete", "Delete an artist. Set delete_files only to also remove files.", r#"{"type":"object","properties":{"artist_id":{"type":"integer"},"delete_files":{"type":"boolean"}},"required":["artist_id"]}"#),
    ("lidarr.artist_update", "Update an artist monitored flag.", r#"{"type":"object","properties":{"artist_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["artist_id"]}"#),
    ("lidarr.albums_list", "List albums, optional artist filter.", r#"{"type":"object","properties":{"artist_id":{"type":"integer"}}}"#),
    ("lidarr.album_get", "Get one album by album ID.", r#"{"type":"object","properties":{"album_id":{"type":"integer"}},"required":["album_id"]}"#),
    ("lidarr.albums_lookup", "Look up album metadata by term.", r#"{"type":"object","properties":{"term":{"type":"string"}},"required":["term"]}"#),
    ("lidarr.album_set_monitored", "Set an album monitored flag.", r#"{"type":"object","properties":{"album_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["album_id"]}"#),
    ("readarr.authors_list", "List Readarr authors.", r#"{"type":"object","properties":{}}"#),
    ("readarr.authors_lookup", "Look up author metadata by term.", r#"{"type":"object","properties":{"term":{"type":"string"}},"required":["term"]}"#),
    ("readarr.author_get", "Get one author by Readarr ID.", r#"{"type":"object","properties":{"author_id":{"type":"integer"}},"required":["author_id"]}"#),
    ("readarr.author_add", "Add an author by foreign ID. Requires quality_profile_id and root_folder_path.", r#"{"type":"object","properties":{"foreign_author_id":{"type":"string"},"quality_profile_id":{"type":"integer"},"root_folder_path":{"type":"string"},"monitored":{"type":"boolean"}},"required":["foreign_author_id","quality_profile_id","root_folder_path"]}"#),
    ("readarr.author_delete", "Delete an author. Set delete_files only to also remove files.", r#"{"type":"object","properties":{"author_id":{"type":"integer"},"delete_files":{"type":"boolean"}},"required":["author_id"]}"#),
    ("readarr.author_update", "Update an author monitored flag.", r#"{"type":"object","properties":{"author_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["author_id"]}"#),
    ("readarr.books_list", "List books, optional author filter.", r#"{"type":"object","properties":{"author_id":{"type":"integer"}}}"#),
    ("readarr.book_get", "Get one book by book ID.", r#"{"type":"object","properties":{"book_id":{"type":"integer"}},"required":["book_id"]}"#),
    ("readarr.books_lookup", "Look up book metadata by term.", r#"{"type":"object","properties":{"term":{"type":"string"}},"required":["term"]}"#),
    ("readarr.book_search", "Trigger BookSearch for a book already in the library.", r#"{"type":"object","properties":{"book_id":{"type":"integer"}},"required":["book_id"]}"#),
    ("readarr.book_set_monitored", "Set a book monitored flag.", r#"{"type":"object","properties":{"book_id":{"type":"integer"},"monitored":{"type":"boolean"}},"required":["book_id"]}"#),
    ("readarr.book_add", "Add one book by foreign ID.", r#"{"type":"object","properties":{"foreign_book_id":{"type":"string"},"title":{"type":"string"},"quality_profile_id":{"type":"integer"},"root_folder_path":{"type":"string"},"monitored":{"type":"boolean"}},"required":["foreign_book_id","quality_profile_id","root_folder_path"]}"#),
    ("readarr.book_delete", "Delete one book.", r#"{"type":"object","properties":{"book_id":{"type":"integer"},"delete_files":{"type":"boolean"}},"required":["book_id"]}"#),
    ("readarr.books_delete_many", "Delete many books; per-item errors collected, never panics.", r#"{"type":"object","properties":{"book_ids":{"type":"array","items":{"type":"integer"}},"delete_files":{"type":"boolean"}},"required":["book_ids"]}"#),
    ("readarr.add_options", "Root folders, quality/metadata profiles, and tags for add flows.", r#"{"type":"object","properties":{}}"#),
    ("bazarr.wanted", "Wanted subtitles: movies, episodes, or all.", r#"{"type":"object","properties":{"kind":{"type":"string","enum":["movies","episodes","all"]}}}"#),
    ("bazarr.search", "Search subtitles by media identifiers.", r#"{"type":"object","properties":{"series_id":{"type":"integer"},"episode_id":{"type":"integer"},"movie_id":{"type":"integer"}}}"#),
    ("bazarr.download", "Download one subtitle by subtitle ID.", r#"{"type":"object","properties":{"subtitle_id":{"type":"string"}},"required":["subtitle_id"]}"#),
    ("bazarr.history", "Subtitle download history.", r#"{"type":"object","properties":{}}"#),
    ("bazarr.providers", "Configured subtitle providers.", r#"{"type":"object","properties":{}}"#),
    ("bazarr.languages", "Supported subtitle languages.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.indexers_list", "List all Prowlarr indexers.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.indexer_get", "Get one indexer by ID.", r#"{"type":"object","properties":{"indexer_id":{"type":"integer"}},"required":["indexer_id"]}"#),
    ("prowlarr.indexer_add", "Add an indexer from a field map.", r#"{"type":"object","properties":{"fields":{"type":"object"}},"required":["fields"]}"#),
    ("prowlarr.indexer_update", "Update an indexer by merging a field map.", r#"{"type":"object","properties":{"indexer_id":{"type":"integer"},"fields":{"type":"object"}},"required":["indexer_id","fields"]}"#),
    ("prowlarr.indexer_delete", "Delete an indexer.", r#"{"type":"object","properties":{"indexer_id":{"type":"integer"}},"required":["indexer_id"]}"#),
    ("prowlarr.indexer_test", "Test one indexer.", r#"{"type":"object","properties":{"indexer_id":{"type":"integer"}},"required":["indexer_id"]}"#),
    ("prowlarr.indexers_test_all", "Test all indexers.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.indexer_schema", "Indexer implementation schema.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.search", "Unified search across all indexers.", r#"{"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}"#),
    ("prowlarr.applications_list", "List synced applications.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.application_get", "Get one application by ID.", r#"{"type":"object","properties":{"app_id":{"type":"integer"}},"required":["app_id"]}"#),
    ("prowlarr.application_sync", "Sync one application to the *arrs.", r#"{"type":"object","properties":{"app_id":{"type":"integer"}},"required":["app_id"]}"#),
    ("prowlarr.applications_sync_all", "Sync all applications.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.application_test", "Test one application.", r#"{"type":"object","properties":{"app_id":{"type":"integer"}},"required":["app_id"]}"#),
    ("prowlarr.history_list", "Indexer history.", r#"{"type":"object","properties":{}}"#),
    ("prowlarr.history_since", "History since a date.", r#"{"type":"object","properties":{"since":{"type":"string"}},"required":["since"]}"#),
    ("prowlarr.history_by_indexer", "History for one indexer.", r#"{"type":"object","properties":{"indexer_id":{"type":"integer"}},"required":["indexer_id"]}"#),
    ("overseerr.requests_list", "List media requests.", r#"{"type":"object","properties":{}}"#),
    ("overseerr.request_get", "Get one request by ID.", r#"{"type":"object","properties":{"request_id":{"type":"integer"}},"required":["request_id"]}"#),
    ("overseerr.request_create", "Create a request: media_type movie|tv plus media_id.", r#"{"type":"object","properties":{"media_type":{"type":"string","enum":["movie","tv"]},"media_id":{"type":"integer"},"seasons":{"type":"array","items":{"type":"integer"}}},"required":["media_type","media_id"]}"#),
    ("overseerr.request_approve", "Approve a request.", r#"{"type":"object","properties":{"request_id":{"type":"integer"}},"required":["request_id"]}"#),
    ("overseerr.request_decline", "Decline a request.", r#"{"type":"object","properties":{"request_id":{"type":"integer"}},"required":["request_id"]}"#),
    ("overseerr.request_delete", "Delete a request.", r#"{"type":"object","properties":{"request_id":{"type":"integer"}},"required":["request_id"]}"#),
    ("overseerr.requests_count", "Request totals.", r#"{"type":"object","properties":{}}"#),
    ("overseerr.requests_pending", "Pending requests only.", r#"{"type":"object","properties":{}}"#),
    ("overseerr.search", "Search media for requesting.", r#"{"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}"#),
    ("overseerr.users_list", "List Overseerr users.", r#"{"type":"object","properties":{}}"#),
    ("overseerr.user_get", "Get one user by ID.", r#"{"type":"object","properties":{"user_id":{"type":"integer"}},"required":["user_id"]}"#),
    ("overseerr.user_requests", "Requests by one user.", r#"{"type":"object","properties":{"user_id":{"type":"integer"}},"required":["user_id"]}"#),
];

fn tool_list_response() -> Value {
    let tools: Vec<Value> = TOOL_CATALOG
        .iter()
        .map(|(name, description, schema)| {
            let schema: Value =
                serde_json::from_str(schema).unwrap_or_else(|_| json!({"type":"object"}));
            json!({"name": name, "description": description, "inputSchema": schema})
        })
        .collect();
    json!({"tools": tools})
}

// --- per-service clients for explicit CLI flags (env/imported/Demo via dispatch_auto otherwise) ---

async fn run_service_command(
    service: &str,
    op_str: &str,
    params: Value,
    base_url: Option<String>,
    api_key: Option<String>,
) -> Result<Value, ArrError> {
    // Explicit flags build a live client directly; otherwise auto-resolve.
    let explicit = match (base_url, api_key) {
        (Some(url), Some(key)) if !url.trim().is_empty() && !key.trim().is_empty() => {
            Some((url, key))
        }
        _ => None,
    };
    let command = to_command(service, op_str, &params).map_err(ArrError::InvalidInput)?;
    if let Some((url, key)) = explicit {
        return run_explicit(service, &url, &key, &command).await;
    }
    dispatch_auto(command).await
}

async fn run_explicit(
    service: &str,
    url: &str,
    key: &str,
    command: &Command,
) -> Result<Value, ArrError> {
    match service {
        "sonarr" => {
            let client = SonarrClient::new(url, key)?;
            arr_core::command::dispatch(&client, command).await
        }
        "radarr" => {
            let client = RadarrClient::new(url, key)?;
            arr_core::command::dispatch_radarr(&client, command).await
        }
        "lidarr" => {
            let client = LidarrClient::new(url, key)?;
            arr_core::command::dispatch_lidarr(&client, command).await
        }
        "readarr" => {
            let client = ReadarrClient::new(url, key)?;
            arr_core::command::dispatch_readarr(&client, command).await
        }
        "bazarr" => {
            let client = BazarrClient::new(url, key)?;
            arr_core::command::dispatch_bazarr(&client, command).await
        }
        "prowlarr" => {
            let client = ProwlarrClient::new(url, key)?;
            arr_core::command::dispatch_prowlarr(&client, command).await
        }
        "overseerr" => {
            let client = OverseerrClient::new(url, key)?;
            arr_core::command::dispatch_overseerr(&client, command).await
        }
        _ => Err(ArrError::InvalidInput(format!(
            "unknown service '{service}'"
        ))),
    }
}

// --- MCP 2024-11-05 Stdio JSON-RPC Loop ---

async fn run_mcp_stdio() -> Result<(), ArrError> {
    eprintln!("rust-arr-mcp: running MCP 2024-11-05 stdio server");
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("rust-arr-mcp stdio read error: {e}");
                break;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err_res = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {e}") }
                });
                let mut out = stdout.lock();
                let _ = writeln!(out, "{err_res}");
                let _ = out.flush();
                continue;
            }
        };

        // Echoed verbatim: numeric and string IDs both round-trip (id_to_key lesson).
        let req_id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");

        let resp = match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "serverInfo": {
                            "name": "rust-arr-mcp",
                            "version": env!("CARGO_PKG_VERSION")
                        },
                        "capabilities": {
                            "tools": { "listChanged": false }
                        }
                    }
                })
            }
            "notifications/initialized" => continue,
            "ping" => json!({ "jsonrpc": "2.0", "id": req_id, "result": {} }),
            "tools/list" => {
                let list = tool_list_response();
                json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "result": list
                })
            }
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or_else(|| json!({}));
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match call_tool(name, args).await {
                    Ok(val) => {
                        let text =
                            serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string());
                        json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "content": [{ "type": "text", "text": text }]
                            }
                        })
                    }
                    Err(e) => {
                        json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "isError": true,
                                "content": [{ "type": "text", "text": format!("Error: {e}") }]
                            }
                        })
                    }
                }
            }
            unknown => {
                json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: '{unknown}'")
                    }
                })
            }
        };

        let mut out = stdout.lock();
        let _ = writeln!(out, "{resp}");
        let _ = out.flush();
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    install_forensic_panic_hook();
    let cli = Cli::parse();

    let emit = |val: &Value, compact: bool| {
        if compact {
            println!("{val}");
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(val).unwrap_or_else(|_| val.to_string())
            );
        }
    };

    match cli.command {
        Commands::Status => match dispatch_auto(Command::Status).await {
            Ok(val) => emit(&val, cli.json),
            Err(e) => {
                eprintln!("rust-arr-mcp error: {e}");
                std::process::exit(1);
            }
        },
        Commands::Sonarr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "sonarr",
                op_str::<SonarrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Radarr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "radarr",
                op_str::<RadarrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Lidarr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "lidarr",
                op_str::<LidarrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Readarr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "readarr",
                op_str::<ReadarrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Bazarr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "bazarr",
                op_str::<BazarrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Prowlarr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "prowlarr",
                op_str::<ProwlarrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Overseerr {
            op,
            params_json,
            base_url,
            api_key,
        } => {
            run_cli(
                "overseerr",
                op_str::<OverseerrOp>(op),
                &params_json,
                base_url,
                api_key,
                cli.json,
            )
            .await;
        }
        Commands::Health { probe } => {
            if probe {
                // One bounded read per configured service (read-only GETs).
                let probes = vec![
                    (
                        "sonarr",
                        dispatch_auto(Command::SonarrSeriesList).await.map(|_| ()),
                    ),
                    (
                        "radarr",
                        dispatch_auto(Command::RadarrMoviesList).await.map(|_| ()),
                    ),
                    (
                        "lidarr",
                        dispatch_auto(Command::LidarrArtistsList { mb_id: None })
                            .await
                            .map(|_| ()),
                    ),
                    (
                        "readarr",
                        dispatch_auto(Command::ReadarrAuthorsList).await.map(|_| ()),
                    ),
                    (
                        "bazarr",
                        dispatch_auto(Command::BazarrProviders).await.map(|_| ()),
                    ),
                    (
                        "prowlarr",
                        dispatch_auto(Command::ProwlarrIndexersList)
                            .await
                            .map(|_| ()),
                    ),
                    (
                        "overseerr",
                        dispatch_auto(Command::OverseerrRequestsCount)
                            .await
                            .map(|_| ()),
                    ),
                ];
                let mut out = serde_json::Map::new();
                for (service, result) in probes {
                    out.insert(service.to_string(), json!({"live": result.is_ok()}));
                }
                emit(&Value::Object(out), cli.json);
            } else {
                emit(&health_snapshot().await, cli.json);
            }
        }
        Commands::Help => {
            emit(&help_snapshot(), cli.json);
        }
        Commands::Mcp => {
            if let Err(e) = run_mcp_stdio().await {
                eprintln!("rust-arr-mcp stdio error: {e}");
                std::process::exit(1);
            }
        }
    }
}

async fn run_cli(
    service: &str,
    op: &str,
    params_json: &str,
    base_url: Option<String>,
    api_key: Option<String>,
    compact: bool,
) {
    let params: Value =
        serde_json::from_str(params_json).unwrap_or_else(|_| Value::Object(Default::default()));
    match run_service_command(service, op, params, base_url, api_key).await {
        Ok(val) => {
            if compact {
                println!("{val}");
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string())
                );
            }
        }
        Err(e) => {
            eprintln!("rust-arr-mcp error: {e}");
            std::process::exit(1);
        }
    }
}

fn op_str<T: OpName>(op: T) -> &'static str {
    op.name()
}

trait OpName {
    fn name(&self) -> &'static str;
}

macro_rules! op_names {
    ($ty:ty, $($variant:ident => $str:expr),* $(,)?) => {
        impl OpName for $ty {
            fn name(&self) -> &'static str {
                match self {
                    $(<$ty>::$variant => $str,)*
                }
            }
        }
    };
}

op_names!(SonarrOp,
    SeriesList => "series_list", SeriesLookup => "series_lookup", SeriesGet => "series_get",
    SeriesAdd => "series_add", SeriesSearch => "series_search", SeriesDelete => "series_delete",
    SeriesUpdate => "series_update", EpisodesList => "episodes_list", EpisodeGet => "episode_get",
    EpisodeSearch => "episode_search", EpisodeSetMonitored => "episode_set_monitored",
);
op_names!(RadarrOp,
    MoviesList => "movies_list", MoviesLookup => "movies_lookup", MovieGet => "movie_get",
    MovieAdd => "movie_add", MovieSearch => "movie_search", MovieDelete => "movie_delete",
    MovieUpdate => "movie_update", MoviesImport => "movies_import",
);
op_names!(LidarrOp,
    ArtistsList => "artists_list", ArtistsLookup => "artists_lookup", ArtistGet => "artist_get",
    ArtistAdd => "artist_add", ArtistDelete => "artist_delete", ArtistUpdate => "artist_update",
    AlbumsList => "albums_list", AlbumGet => "album_get", AlbumsLookup => "albums_lookup",
    AlbumSetMonitored => "album_set_monitored",
);
op_names!(ReadarrOp,
    AuthorsList => "authors_list", AuthorsLookup => "authors_lookup", AuthorGet => "author_get",
    AuthorAdd => "author_add", AuthorDelete => "author_delete", AuthorUpdate => "author_update",
    BooksList => "books_list", BookGet => "book_get", BooksLookup => "books_lookup",
    BookSearch => "book_search", BookSetMonitored => "book_set_monitored", BookAdd => "book_add",
    BookDelete => "book_delete", BooksDeleteMany => "books_delete_many", AddOptions => "add_options",
);
op_names!(BazarrOp,
    Wanted => "wanted", Search => "search", Download => "download",
    History => "history", Providers => "providers", Languages => "languages",
);
op_names!(ProwlarrOp,
    IndexersList => "indexers_list", IndexerGet => "indexer_get", IndexerAdd => "indexer_add",
    IndexerUpdate => "indexer_update", IndexerDelete => "indexer_delete", IndexerTest => "indexer_test",
    IndexersTestAll => "indexers_test_all", IndexerSchema => "indexer_schema", Search => "search",
    ApplicationsList => "applications_list", ApplicationGet => "application_get",
    ApplicationSync => "application_sync", ApplicationsSyncAll => "applications_sync_all",
    ApplicationTest => "application_test", HistoryList => "history_list", HistorySince => "history_since",
    HistoryByIndexer => "history_by_indexer",
);
op_names!(OverseerrOp,
    RequestsList => "requests_list", RequestGet => "request_get", RequestCreate => "request_create",
    RequestApprove => "request_approve", RequestDecline => "request_decline",
    RequestDelete => "request_delete", RequestsCount => "requests_count",
    RequestsPending => "requests_pending", Search => "search", UsersList => "users_list",
    UserGet => "user_get", UserRequests => "user_requests",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_82_tools() {
        assert_eq!(TOOL_CATALOG.len(), 82);
    }

    #[tokio::test]
    async fn unknown_tool_becomes_error_content_not_panic() {
        let result = call_tool("nope.nothing", json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn legacy_portmanteau_alias_routes_correctly() {
        // All three forms must resolve identically in ANY environment
        // (live [] on configured machines, Demo fixtures elsewhere).
        let val = call_tool("sonarr_series", json!({"operation": "list"}))
            .await
            .unwrap_or(Value::Null);
        let val2 = call_tool("sonarr.series_list", json!({}))
            .await
            .unwrap_or(Value::Null);
        let val3 = call_tool("sonarr_series_list", json!({}))
            .await
            .unwrap_or(Value::Null);
        assert!(val.is_array() && val2.is_array() && val3.is_array());
        assert_eq!(val, val2);
        assert_eq!(val2, val3);
    }

    #[tokio::test]
    async fn mcp_round_trip_resolves() {
        let val = call_tool("radarr.movies_list", json!({}))
            .await
            .unwrap_or(Value::Null);
        assert!(val.is_array());
    }

    #[tokio::test]
    async fn write_safety_rejects_missing_values() {
        // add without required ids → InvalidInput, never a silent default add.
        let result = call_tool("radarr.movie_add", json!({"title": "x"})).await;
        assert!(result.is_err());
        let result = call_tool("sonarr.series_delete", json!({})).await;
        assert!(result.is_err());
    }
}
