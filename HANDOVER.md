# HANDOVER — `rust-arr-mcp`

- **Being worked on:** Phase 2c done — `radarr.rs` + per-service routing, 18 tests green (Shadow-Then-Swap).
- **Last concrete action (2026-10-18):** Phase 2c `radarr.rs` + 8 `Command` variants + routing, 18 tests green. Observability proposal filed in mcp-central-docs.
- **Next step:** Phase 2d `lidarr.rs` + `readarr.rs`, or Phase 3 L2 `mcp` stdio — Sandra's call.
- **Blockers:** None. Legacy `arr-mcp@3e907b6` clean; `rust-plex-mcp` hands-off (agy).
- **Deliberately not done:** Remaining 5 drivers (lidarr/readarr/bazarr/prowlarr/overseerr), L2 `mcp` stdio (Phase 3), L3 dashboard (Phase 4), `CHANGELOG.md` (Phase 2 closeout).
