# HANDOVER — `rust-arr-mcp`

- **Being worked on:** Phase 2b done — `sonarr.rs` + command bus, 13 tests green (Shadow-Then-Swap).
- **Last concrete action (2026-10-18):** Phase 2b `sonarr.rs` + `command.rs` + L2 wiring, 13 tests green. Live Sonarr reachable (empty library); tests pinned to Demo.
- **Next step:** Phase 2c `radarr.rs` (8 ops, same pattern) or Phase 3 L2 `mcp` stdio — Sandra's call.
- **Blockers:** None. Legacy `arr-mcp@3e907b6` clean; `rust-plex-mcp` hands-off (agy).
- **Deliberately not done:** Remaining 6 drivers (radarr/lidarr/readarr/bazarr/prowlarr/overseerr), L2 `mcp` stdio (Phase 3), L3 dashboard (Phase 4), `CHANGELOG.md` (Phase 2 closeout).
