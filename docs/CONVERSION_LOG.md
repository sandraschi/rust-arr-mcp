# CONVERSION_LOG — `rust-arr-mcp`

## 2026-10-18 — Phase 0: dossier gate (forensic recon, no `.rs` written)

- Legacy `arr-mcp@3e907b6` (`master`, clean). 15 tool modules inventoried (sonarr/radarr/lidarr/readarr×3/bazarr/prowlarr/overseerr/cross/health/help/agentic/prefab). Ports `:10938/:10939`; Arr hosts `:7878/:8989/:8686/:9696/:8787/:5055(Overseerr off)`. 18 webapp pages enumerated. `native/` = Tauri shell → Bucket A.
- Wrote `docs/CONVERSION_DOSSIER.md` (7 points), this log, `HANDOVER.md`, `.gitignore` (before first add).
- Next: `git init` + first commit `docs(phase-0)`, then Phase 1 mechanical stamp (kernel from `rust-devices-mcp`, operation literals via live `tools/list`).
- Legacy fence: `arr-mcp` untouched (status clean before/after recon). `rust-plex-mcp` untouched (agy owns it).

## 2026-10-18 — Phase 1: mechanical workspace stamp (Checkpoint 1 green)

- Stamped from `rust-plex-mcp` (3 crates, no store; `egui` L3): root `Cargo.toml` (plex deps + `serde_yaml` for config.yaml import), `arr-core`/`arr-mcp`/`arr-app` manifests, `lib.rs` (`Command::Status` + `dispatch_future_is_send` + round-trip), `error.rs` (`ArrError` 6 variants), L2 `status --json`, L3 placeholder, `justfile`, `AGENTS.md`.
- Gates: `cargo check --workspace` ok (63s, eframe build); `cargo clippy --workspace --all-targets --all-features -- -D warnings` green (fixed `expect()` in own test → `if let`); `cargo test --workspace` 2 passed; `cargo fmt --all --check` green.
- Next: Phase 2 L1 drivers (sonarr/radarr/lidarr/readarr/bazarr/prowlarr/overseerr + `import.rs` + `Command` wiring). Operation literals via live legacy `tools/list` appended to dossier §4 before L1 code.

## 2026-10-18 — Phase 2a: op-table lock + `import.rs` (6 tests green)

- Dossier §4-appendix: 21 `Literal[` sets / ~80 variants (code truth, grep). Correction: arr tools are atomic portmanteaus per service (not single-op); prefab cards UI-only → `egui`.
- `import.rs`: sibling `../arr-mcp` discovery, dotenv parse + `${VAR:-default}`/`$VAR` expansion, placeholder-key rejection (`your-*`), `config/arr-services.json` override, resolution order explicit > env > vault > imported. 4 unit tests. Fixed `expect()`→`if let`, let-chains→nested (ed2021), `if_same_then_else`, `unnecessary_get_then_check`.
- Gates: clippy `-D warnings` green, `cargo test` 6 passed (2 lib + 4 import), fmt green.
- Next: Phase 2b `sonarr.rs` (series 7 ops + episodes 4 ops) + `Command` wiring + live-read probe.

## 2026-10-18 — Phase 2b: `sonarr.rs` driver + command bus (13 tests green)

- `sonarr.rs` (~450 lines): `/api/v3`, `X-Api-Key`, 30 s timeout, 32 MiB cap; series list/lookup/get/add/search/delete/update + episodes list/get/search/set_monitored with legacy guards (required fields, id-mismatch blocks search, season-existence check). `Demo` fixtures, offline tests. Fixed ed2021 let-chains, `if_same_then_else`, test lints.
- `command.rs`: 12 `Command` variants + async `dispatch(client, cmd)` + `dispatch_auto` (env → imported `arr-services.json` → Demo). L2 `status` via `dispatch_auto`.
- Live oracle (accidental, read-only): Goliath Sonarr reachable at `localhost:8989` with real key from `arr-services.json` — `series_list` returned `[]` (auth OK, empty library). Pinned `command.rs` tests to explicit Demo clients afterward (deterministic CI).
- Gates: clippy `-D warnings` green, `cargo test` 13 passed (6 sonarr + 4 import + 3 command), fmt green.
- Next: Phase 2c `radarr.rs` (8 ops, same pattern) or L2 `mcp` stdio (Phase 3) — your call.

## 2026-10-18 — Phase 2c: `radarr.rs` driver (18 tests green)

- `radarr.rs`: `/api/v3` + `X-Api-Key` + 30 s + 32 MiB; list/lookup/get/add/search/delete/update/import (`MoviesSearch` + `movieIds`, import via `/manualimport`), same guards as sonarr. Demo fixtures, 4 offline tests.
- `command.rs`: 8 `Radarr*` variants + `dispatch_radarr` + per-service `dispatch_auto` routing (radarr vs sonarr) + `resolve_radarr_client` (`RADARR_URL/API_KEY` → imported → Demo). `id_to_key` lesson from Gemini interceptor doc recorded for Phase 3 (accept num-or-string IDs).
- Gates: clippy `-D warnings` green, `cargo test` 18 passed (13 prior + 4 radarr + 1 command), fmt green.
- Also filed `mcp-central-docs/research/FLEET_MCP_OBSERVABILITY.md` (interceptor belongs in fleet lane, not in-repo).
- Next: Phase 2d `lidarr.rs` + `readarr.rs`, or Phase 3 L2 `mcp` stdio — your call.

## 2026-10-18 - Phase 2d: lidarr.rs + readarr.rs (26 tests green)

- lidarr.rs (/api/v1): artists list/lookup/get/add/delete/update (foreignArtistId) + albums list/get/lookup/set_monitored (GET+PUT). Demo fixtures, 3 offline tests.
- readarr.rs (/api/v1): authors 6 + books list/get/lookup/search/set_monitored (BookSearch reconstructed Servarr shape, flagged for live verify) + mutations add/delete/delete_many + add_options (4 GETs aggregated). Demo fixtures, 3 offline tests.
- command.rs: +25 variants, dispatch_lidarr/dispatch_readarr, shared resolve_service_config helper, per-service auto-routing, explicit-Demo tests. Exhaustive match caught missing cross-arms (E0004) - fixed same way.
- Gates: clippy -D warnings green, cargo test 26 passed (18 + 3 + 3 + 2), fmt green.
- Next: Phase 2e bazarr.rs + prowlarr.rs + overseerr.rs, then Phase 3 L2 mcp stdio (cross/health compose there).

## 2026-10-18 - Phase 2e: bazarr.rs + prowlarr.rs + overseerr.rs (38 tests green)

- bazarr.rs (/api): wanted (movies/episodes/all), search, download, history, providers, languages. 6 ops, 3 offline tests. Subtitle search/download shapes reconstructed - pending live verify.
- prowlarr.rs (/api/v1): indexers 8 (incl. test/testall/schema), unified search, applications 5, history 3. 17 ops, 3 offline tests. App sync/test + history-since/by-indexer shapes reconstructed - pending live verify.
- overseerr.rs (/api/v1 absolute): requests 8 (incl. approve/decline/count/pending), search, users 3. 12 ops, 3 offline tests. Create payload reconstructed - pending live verify.
- command.rs: 80 variants total (Status + 11+8+10+15+6+17+12), dispatch_bazarr/prowlarr/overseerr, resolvers, routing, explicit-Demo tests. All 7 drivers wired.
- Gates: clippy -D warnings green, cargo test 38 passed (26 + 12), fmt green.
- Next: Phase 3 L2 mcp stdio (cross/health/help compose over the bus, dual aliases, id_to_key, byte-exact probe).

## 2026-10-18 - Phase 3: L2 CLI + dual-aliased MCP stdio (43 tests green)

- main.rs (~1400 lines): per-service CLI subs (typed --op enums + --params-json + --base-url/--api-key, --json everywhere; documented deviation from 80 one-shot subs), panic hook (crash.log, caught a real clap help-collision in-probe), 82-tool catalog, 3-form aliases (dot + underscore + legacy portmanteau w/ explicit op table; orchestration-layer legacy tools get a clear not-yet-ported error, not silent misroute), run_mcp_stdio mirror (verbatim id echo incl. string IDs, -32700/-32601, notifications skip, isError content).
- Health split (documented): MCP arr.health = config presence (no network on stdio); CLI --probe does one bounded read per service. arr.help = inventory. orchestrate/calendar/stats/agentic/prefab deferred (need unported endpoints).
- Probe (byte-exact, live): initialize/tools-list(82)/help/legacy-list(-32601/-32700, notifications skipped). Legacy sonarr_series list hit LIVE Sonarr -> [] (auth OK, empty library). String-ID echo verified.
- Contract tests (5): catalog-82, unknown->err-not-panic, legacy-alias-equality (env-proof), round-trip, write-safety rejects.
- Gates: clippy -D warnings green, cargo test 43 passed (38 + 5), fmt green.
- Next: Phase 4 L3 egui dashboard (18 pages) or CHANGELOG + release v0.1.0.

## 2026-10-18 - Phase 4: L3 egui dashboard, 19 pages (48 tests green)

- app.rs (~1100 lines): 19 Page variants in 5 groups mirroring webapp/src/pages, mpsc Job bus + pending counter + repaint-while-pending, theme toggle, zoom (sampled/applied outside ctx.input), status line, per-service tables + lookup, health (bounded live reads), chat (OpenAI-compatible POST via runtime), settings (fleet LLM card + per-service vault-backed creds + pinned profiles), logs/inbox/help/tools/raw/inspector/orchestrate-router (no dead buttons - every button navigates, spawns, saves, or toggles).
- main.rs/vault.rs/registry.rs mirrored from rust-calibre-mcp (SERVICE/paths renamed; icon loading dropped - no icons dir; registry holds name+URL pins, secrets in vault).
- API migration (honest): eframe 0.37 removed update(ctx)+SidePanel/TopBottomPanel/CentralPanel(ctx) and Frame::none - ported to fn ui + Panel::top/left/bottom/central + Frame::NONE (same as calibre). Caught E0004/E0521/E0599 class errors via compiler.
- Glyph hygiene: em-dash/arrow offenders replaced with ASCII (allowlist holds).
- Gates: clippy -D warnings green, cargo test 48 passed (38 core + 5 mcp + 5 app incl. glyph scan + kittest), fmt green, --smoke OK (status + series/movies arrays).
- Next: CHANGELOG.md + release v0.1.0, then cross-service compose (orchestrate/calendar/stats endpoints) as follow-up tiers.
