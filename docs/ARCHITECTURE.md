# Architecture — `rust-arr-mcp`

ArtCraft 3-tier workspace. Zero-hop: L2/L3 call L1 in-process. No subprocess
wrapping, no ports, no Python, no Node. MCP is stdio-only; *arr access is
outbound HTTPS.

```
rust-arr-mcp/
├── crates/arr-core/      L1: 7 drivers + import + command bus (no UI/MCP deps)
│   └── src/: lib.rs, command.rs (80 variants), error.rs, import.rs,
│             sonarr.rs, radarr.rs, lidarr.rs, readarr.rs,
│             bazarr.rs, prowlarr.rs, overseerr.rs
├── crates/arr-mcp/       L2: clap CLI + MCP 2024-11-05 stdio (main.rs)
└── crates/arr-app/       L3: egui dashboard (app.rs 19 pages, vault.rs, registry.rs)
```

## Data flow

- **CLI/MCP → `dispatch_auto(cmd)`** → per-service resolver (explicit flags >
  env > imported legacy checkout > Demo) → typed driver method → `X-Api-Key`
  HTTPS with 30 s timeout + 32 MiB cap → JSON.
- **Dashboard → `dispatch_auto` via `spawn()`** (tokio runtime, mpsc `Job` bus,
  pending counter + repaint). The UI thread never touches network, disk, or
  keyring.
- **Stdio purity**: stdout carries JSON-RPC only; every log goes to stderr.
  Request IDs echo verbatim (numeric and string).

## Protocols

- Sonarr/Radarr/Lidarr/Readarr/Prowlarr: Servarr `X-Api-Key` header,
  `/api/v3` (Sonarr/Radarr) or `/api/v1` (others).
- Bazarr: `/api` + `X-Api-Key`.
- Overseerr: absolute `/api/v1/*` paths + `X-Api-Key`.
- MCP: JSON-RPC 2.0 line-delimited stdio, protocol version `2024-11-05`.

## Naming (triple alias)

Every tool answers to three forms: `sonarr.series_list` (canonical dot),
`sonarr_series_list` (underscore), legacy `sonarr_series(operation="list")`
(zero-edit migration). Cross-service orchestration tools (`arr_orchestrate`,
`arr_calendar`, `arr_stats`, `arr_agentic`) and prefab cards are intentionally
unported (need unported endpoints) — calling one returns a clear not-ported
error, never a silent misroute.
