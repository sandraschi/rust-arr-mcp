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
