# CONVERSION_DOSSIER — `arr-mcp` → `rust-arr-mcp` (Phase 0 gate)

**Date:** 2026-10-18 · **Legacy baseline:** `arr-mcp@3e907b6` (`master`, tree clean) · **Mode:** Shadow-Then-Swap (daily driver, keep legacy running)

## 1. Bucket & hidden binary / privilege-split audit — Bucket A Pure Rust

- `native/` = Tauri shell (`binaries/capabilities/gen/icons/resources/src/target/windows`, standard wrapper). `src-tauri/` present (same). No install-once elevated `LocalSystem` service, no named pipe, no host-app plugin (`bpy`/`pcbnew`), no `.wasm` stub. `service/` absent.
- Verdict: **Bucket A (REST, pure `reqwest` + `serde`)**. Highest ROI, zero obscure crates.

## 2. Live service, port & database contention audit

- Legacy MCP: backend `:10938` → `/api/health`, frontend `:10939` (`fleet-start.config.ps1`, module-serve `arr_mcp --http --port 10938`).
- Arr stack (`.env.example`): Radarr `:7878`, Sonarr `:8989`, Lidarr `:8686`, Prowlarr `:9696`, Readarr `:8787`, Overseerr `:5055` (disabled by default). Bazarr port via `bazarr_tools` (docs pass).
- No shared SQLite / lock files / COM ports. Rust shadow touches the network only (`reqwest` to the Arr hosts above); legacy `D:\Dev\repos\arr-mcp` stays running untouched (read-only fence verified by SHA + status before/after every phase).

## 3. Legacy git baseline snapshot

- `arr-mcp@3e907b6`, branch `master`, `status --porcelain` clean at recon (2026-10-18). Re-verify before each phase; any drift aborts.

## 4. 1:1 MCP tool mapping (module level — operation literals locked in Phase 1 via live `tools/list`)

| Legacy module (`src/arr_mcp/tools/`) | Representative tools | Rust `Command` (planned) |
|---|---|---|
| `sonarr_tools.py` | `sonarr_series`, `sonarr_episodes` | `sonarr.series`, `sonarr.episodes` |
| `radarr_tools.py` | `radarr_movies` | `radarr.movies` |
| `lidarr_tools.py` | `lidarr_artists`, `lidarr_albums` | `lidarr.artists`, `lidarr.albums` |
| `readarr_tools.py` + `readarr_options.py` + `readarr_book_mutations.py` | `readarr_authors`, `readarr_books` (+ mutations) | `readarr.authors`, `readarr.books`, `readarr.mutate` |
| `bazarr_tools.py` | `bazarr_subtitles` | `bazarr.subtitles` |
| `prowlarr_tools.py` | `prowlarr_indexers/search/applications/history` | `prowlarr.*` (4) |
| `overseerr_tools.py` | `overseerr_requests/search/users` | `overseerr.*` (3) |
| `cross_arr_tools.py` | `arr_orchestrate/calendar/stats` (+ `_stack_status/summary/disk/queue/history`) | `arr.orchestrate/calendar/stats` |
| `health_tools.py` | `arr_health`, `arr_shutdown` | `arr.health` (+ `status`, read-only; shutdown = explicit-confirm fence) |
| `help_tools.py` / `agentic.py` / `prefab_tools.py` | `arr_help`, `arr_agentic`, cards (`arr_health_card/calendar_card/stats_card`) | `arr.help`, `bridge.*`, `llm.*`; cards become `egui` panels, not MCP tools |
| L2 aliases | — | Dual aliasing: `arr.series` + `arr_series` + legacy `sonarr_series(operation="…")` (contract §8) |

Operation-literal table (every `operation="…"` variant per portmanteau) to be extracted from the running legacy server (`tools/list`) in Phase 1 and appended here before any L1 code — gate requirement, not skipped.

## 5. Credential / `.env` / config inventory (`import.rs` spec)

From `.env.example` (values redacted, presence verified): `RADARR_URL/API_KEY/ENABLED`, `SONARR_*`, `LIDARR_*`, `PROWLARR_*`, `READARR_*`, `OVERSEERR_*(ENABLED=false)`. Resolution order: explicit args > process env > OS vault (`vault.rs`, `SERVICE="rust-arr-mcp"`) > imported `../arr-mcp/.env`. `import.rs` parses `${VAR:-default}` expansions; unit-tested fallback chain.

## 6. Write-safety & destructive-action fence

Mutating ops (add/delete series/movie/book, library mutations, `arr_shutdown`, indexers/applications edits) get `readonly: bool` (default true for query commands) + explicit confirmation flags; shutdown requires confirm. No destructive default. Offline/missing-creds → `ArrError::Offline/Auth` diagnostic + `Demo` fallback, never crash (contract §9).

## 7. L3 `egui` structural parity & clone plan (18 pages)

Legacy `webapp/src/pages`: `Dashboard`, `SonarrPage`, `RadarrPage`, `LidarrPage`, `ReadarrPage`, `BazarrPage`, `ProwlarrPage`, `OverseerrPage`, `ServicePages`, `OrchestratePage`, `InspectorPage`, `InboxPage`, `ChatPage`, `ToolsPage`, `SkillsPage`, `LoggerPage`, `SettingsPage` (+ `Settings` LLM card per contract: Ollama/LMStudio/OpenAI/Anthropic/Custom, `gemma4:12b` default, keyring-backed keys, elicitation + test pill), `HelpPage`. Each gets a typed `Page` variant; golden PNGs locked in Phase 4 (Dark + Light).
