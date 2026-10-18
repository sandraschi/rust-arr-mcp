# Changelog

All notable changes to `rust-arr-mcp` are documented in this file.

## [0.1.1] - 2026-10-18

Patch: release stack, CI, L3 help/UX polish. No wire changes, no new tools.

### Added

- Rust CI (`.github/workflows/ci.yml`, Windows-only: clippy, test, fmt, smoke —
  no ruff/biome/pytest), `just ci` (+ `clippy`/`fmt`/`smoke` recipes; justfile
  de-plexed — it invoked `rust-plex-mcp` binaries), `docs/RUST.md` (toolchain,
  deny gate, eframe 0.37 notes, release flow).
- L3 Help page tabs: Overview / Rust stack / MCP + CLI.
- Service heroes on all 7 pages (3 lines: what + standard flow + safety).
- Topbar sun/moon toggle (vector-drawn, zero glyphs, mirrored from
  `rust-devices-mcp`).

### Fixed

- `disable_help_subcommand`: clap's auto-`help` collided with our Help page
  variant (forensic hook caught it live in-probe).

## [0.1.0] - 2026-10-18

Initial ArtCraft conversion of `arr-mcp` (`master@3e907b6`) under the `repconv`
standard (Phases 0-4, Shadow-Then-Swap; legacy repo untouched, read-only fence
verified before and after every phase).

### Added

- **3-Crate ArtCraft Architecture (`arr-core` L1, `rust-arr-mcp` L2, `arr-app` L3)**:
  workspace `Cargo.toml` (plex-derived deps + `serde_yaml`), deny-all lint gate
  (`unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`), `Cargo.lock`
  committed, `justfile`, `AGENTS.md`.
- **80 Commands across 7 Arr services** (`arr-core::Command`, one typed variant
  per legacy op): Sonarr 11 (series 7 + episodes 4), Radarr 8, Lidarr 10
  (artists 6 + albums 4), Readarr 15 (authors 6 + books 5 + mutations 3 +
  add_options), Bazarr 6, Prowlarr 17 (indexers 8 + search + applications 5 +
  history 3), Overseerr 12 (requests 8 + search + users 3).
- **Zero-config legacy import (`import.rs`)**: sibling `../arr-mcp` discovery,
  dotenv parse with `${VAR:-default}`/`$VAR` expansion, `your-*` placeholder
  rejection, `config/arr-services.json` override. Resolution order: explicit
  args > process env > OS vault > imported files. Unit-tested fallback chain.
- **Per-service async dispatch** (`dispatch_*` + `dispatch_auto` with Demo
  fallback): 30 s timeouts, 32 MiB payload caps, `X-Api-Key` auth, typed
  `ArrError` (InvalidInput/Offline/Timeout/Auth/Protocol/Unsupported), legacy
  guards ported (required fields, id-mismatch blocks search, season-existence
  check, GET-then-PUT updates). `Demo` fixtures for all 7 services; tests are
  hermetic (explicit Demo clients) after a live-config false failure taught us
  the lesson once.
- **Dual-aliased MCP 2024-11-05 server (`rust-arr-mcp mcp`)**: 82-tool catalog
  with correct JSON schemas; three accepted name forms (`sonarr.series_list`,
  `sonarr_series_list`, legacy `sonarr_series(operation="list")` via an explicit
  op table); verbatim request-ID echo (numeric and string); `-32700`/`-32601`
  handling; notifications skipped; `isError` content (never panics).
  Byte-exact live probe recorded in `docs/CONVERSION_LOG.md` (init, list=82,
  help, legacy live list, bogus, garbage, string-ID echo).
- **Per-service CLI** (`status`, 7 service subs with typed `--op` enums +
  `--params-json`, `health [--probe]`, `help`, `mcp`; `--json` everywhere):
  documented deviation from 80 one-shot subcommands (boilerplate vs envelope,
  rationale in log). Forensic panic hook (`%LOCALAPPDATA%\rust-arr-mcp\crash.log`,
  caught a real clap `help`-subcommand collision in-probe).
- **L3 `egui` dashboard (`arr-app`, 19 pages in 5 groups mirroring
  `webapp/src/pages`)**: mpsc job bus (zero UI-thread I/O), theme/zoom/status,
  per-service tables + lookup, bounded live health, OpenAI-compatible chat,
  fleet LLM card (5 providers, elicitation, vault keys, connection test),
  per-service vault credentials + pinned profiles, logs/inbox/help/tools/raw/
  inspector, honest orchestrate router. Zero dead buttons. Ported to the
  eframe 0.37 `fn ui` + `Panel::*` + `Frame::NONE` API. Glyph allowlist holds.
  `--smoke` headless parity check green.
- **Fleet observability proposal** (`mcp-central-docs/research/
  FLEET_MCP_OBSERVABILITY.md`): interceptor supervision belongs in a fleet lane,
  not in-repo; adopted `id_to_key`, handshake-test pattern, `machete`/`audit`
  recipes, `--compact` note.

### Verified live (read-only)

- Goliath Sonarr reachable at `:8989` (real key from `arr-services.json`,
  empty library `[]`, auth OK). No writes were ever issued to any Arr service
  during this conversion.

### Known gaps (follow-up tiers, all flagged in-code and in `HANDOVER.md`)

- Reconstructed wire shapes pending live verify: `BookSearch` command,
  Bazarr search/download params, Prowlarr app sync/test + history-since/by-indexer
  paths, Overseerr create payload.
- Cross-service compose unported: `arr_orchestrate`, `arr_calendar`, `arr_stats`,
  `arr_agentic`, prefab cards (need calendar/system endpoints).
- NSIS installer scaffold pending.
