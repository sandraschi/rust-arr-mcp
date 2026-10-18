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
