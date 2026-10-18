# AGENTS.md - rust-arr-mcp (ArtCraft containment harness, adapted)

Pure-Rust conversion of `arr-mcp` (Sonarr/Radarr/Lidarr/Readarr/Bazarr/Prowlarr/Overseerr:
libraries, media, playlists, users, sessions, requests). Bucket A pure REST. Shadow-Then-Swap;
legacy `D:\Dev\repos\arr-mcp` stays running untouched (read-only fence).

## 1. Safety and error handling (never crash)

- Workspace lint gate (root `Cargo.toml`): `clippy::unwrap_used`, `clippy::expect_used`,
  `clippy::panic`, `clippy::unimplemented`, `clippy::todo` are all `deny`.
- Every fallible function returns `Result<T, ArrError>` and bubbles with `?`. Never `unwrap`/`expect`.
- Hostile input: no `buf[i]` (use `.get()`), `checked_*`/`saturating_*` math, `char`-boundary
  string slicing, `X-Api-Key` present before any socket touch.
- Every HTTP call carries timeout + payload cap (Arr libraries return megabytes; never buffer unbounded).
- Every bug fix ships a regression test that fails before the fix.
- Stdio MCP invariant: **stdout is pure JSON-RPC only**; logs on stderr via `eprintln!`.

## 2. Architecture and seams

- L1 `arr-core`: *arr REST protocols + `Command::dispatch`. Zero UI, zero MCP deps.
- L2 `rust-arr-mcp`: CLI-first (`--json`) + MCP `mcp` stdio (2024-11-05, hand-rolled framing).
  Dual aliasing: `arr.<action>` + `arr_<action>` + legacy portmanteau (`sonarr_series` with
  `args["operation"]`) per `docs/CONVERSION_DOSSIER.md` §4.
- L3 `arr-app`: `egui` dashboard calling `arr-core` in-process. 18 pages mirror
  `webapp/src/pages` (Dashboard, Sonarr, Radarr, Lidarr, Readarr, Bazarr, Prowlarr,
  Overseerr, Services, Orchestrate, Inspector, Inbox, Chat, Tools, Skills, Logger,
  Settings+LLM card, Help).
- No destructive actuation (delete library items, empty trash, shutdown) without explicit
  confirmation flags. Reads are default. Offline/missing-creds → `ArrError::Offline/Auth` + Demo.

## 3. MCP surface contract

- Portmanteau parity: every legacy `operation=` variant in the dossier §4 maps to a `Command`
  variant AND a legacy alias in `call_tool`. Exhaustive `match` catches omissions.
- Short imperative descriptions with units and required params (`series_id`, `movie_id`, `api_key`).
- Notifications get no response. Unknown methods `-32601`, bad params `-32602`, parse failures `-32700`.

## 4. Definition of done (6 gates)

- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` green.
- [ ] `cargo test --workspace` green (incl. `dispatch_future_is_send`).
- [ ] `cargo fmt --all --check` green.
- [ ] Stdio probe `initialize -> tools/list -> tools/call` byte-exact.
- [ ] `Cargo.lock` committed; no `target/`, `*.bak`, `*.exe` in status.
- [ ] `CHANGELOG.md` checkpoint line added (missing as of Phase 1 — Phase 2 adds it).

## 5. Fleet notes

- Windows-first: `just` recipes prefix `$env:USERPROFILE\.cargo\bin`.
- Never hardcode ports, IPs, tokens, paths. Arr hosts resolve: explicit args > env
  (`SONARR_URL/API_KEY`, `RADARR_*`, `LIDARR_*`, `READARR_*`, `PROWLARR_*`, `OVERSEERR_*`,
  `BAZARR_*`) > OS vault > sibling `arr-mcp/.env`. Secrets never logged.
- Shadow mode: never bind `:10938`/`:10939` or Arr ports; MCP is stdio-only, Arr access is
  outbound HTTP. Hands off `rust-plex-mcp` (agy owns it).
