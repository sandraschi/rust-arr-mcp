# Tools — `rust-arr-mcp` MCP reference (82 tools)

Canonical names use dots (`sonarr.series_list`); underscore joins
(`sonarr_series_list`) and legacy portmanteaus (`sonarr_series` +
`operation="list"`) route identically. `arr.status`, `arr.health`, `arr.help`
take no arguments. Full JSON schemas: `tools/list` at runtime.

## Sonarr TV (11)

`series_list`, `series_lookup(term)`, `series_get(series_id)`,
`series_add(tvdb_id, quality_profile_id, root_folder_path, [title, monitored, search_for_missing])`,
`series_search(series_id)`, `series_delete(series_id, [delete_files, add_import_list_exclusion])`,
`series_update(series_id, monitored)`, `episodes_list(series_id, [season_number])`,
`episode_get(episode_id)`, `episode_search(episode_id | series_id+season_number)`,
`episode_set_monitored(episode_id, monitored)`.

## Radarr movies (8)

`movies_list`, `movies_lookup(term)`, `movie_get(movie_id)`,
`movie_add(tmdb_id, quality_profile_id, root_folder_path, …)`,
`movie_search(movie_id)`, `movie_delete(movie_id, …)`, `movie_update(movie_id, monitored)`,
`movies_import([folder])`.

## Lidarr music (10)

`artists_list([mb_id])`, `artists_lookup(term)`, `artist_get(artist_id)`,
`artist_add(foreign_artist_id, quality_profile_id, root_folder_path, …)`,
`artist_delete(artist_id, …)`, `artist_update(artist_id, monitored)`,
`albums_list([artist_id])`, `album_get(album_id)`, `albums_lookup(term)`,
`album_set_monitored(album_id, monitored)`.

## Readarr books (15)

`authors_list`, `authors_lookup(term)`, `author_get(author_id)`,
`author_add(foreign_author_id, quality_profile_id, root_folder_path, …)`,
`author_delete(author_id, …)`, `author_update(author_id, monitored)`,
`books_list([author_id])`, `book_get(book_id)`, `books_lookup(term)`,
`book_search(book_id)`, `book_set_monitored(book_id, monitored)`,
`book_add(foreign_book_id, …)`, `book_delete(book_id, …)`,
`books_delete_many(book_ids[], …)`, `add_options()`.

## Bazarr subtitles (6)

`wanted(kind: movies|episodes|all)`, `search(series_id?, episode_id?, movie_id?)`,
`download(subtitle_id)`, `history`, `providers`, `languages`.

## Prowlarr indexers (17)

`indexers_list`, `indexer_get(indexer_id)`, `indexer_add(fields{})`,
`indexer_update(indexer_id, fields{})`, `indexer_delete(indexer_id)`,
`indexer_test(indexer_id)`, `indexers_test_all`, `indexer_schema`,
`search(query)`, `applications_list`, `application_get(app_id)`,
`application_sync(app_id)`, `applications_sync_all`, `application_test(app_id)`,
`history_list`, `history_since(since)`, `history_by_indexer(indexer_id)`.

## Overseerr requests (12)

`requests_list`, `request_get(request_id)`,
`request_create(media_type: movie|tv, media_id, [seasons[]])`,
`request_approve(request_id)`, `request_decline(request_id)`,
`request_delete(request_id)`, `requests_count`, `requests_pending`,
`search(query)`, `users_list`, `user_get(user_id)`, `user_requests(user_id)`.

## Stack (3)

`arr.status` (no network), `arr.health` (config presence; CLI `--probe` does
bounded live reads), `arr.help` (inventory).

## Not ported (clear error, never silent)

`arr_orchestrate`, `arr_calendar`, `arr_stats`, `arr_agentic`, prefab cards —
need unported endpoints; tracked in `HANDOVER.md`.
