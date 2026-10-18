# HANDOVER — `rust-arr-mcp`

- **Being worked on:** Phase 2a done — op-table locked, `import.rs` green (repconv `arr-mcp` → `rust-arr-mcp`, Shadow-Then-Swap).
- **Last concrete action (2026-10-18):** Phase 1 stamp (Checkpoint 1 green) + dossier §4-appendix (21 Literal sets) + `import.rs` (dotenv/expansion/resolution, 4 tests). Gates: clippy/test-6/fmt green.
- **Next step:** Phase 2b `sonarr.rs` (series 7 ops + episodes 4 ops, `X-Api-Key` + timeout + cap + Demo) + `Command` wiring + live-read probe (real Sonarr if configured, else Demo fixtures).
- **Blockers:** None. Legacy `arr-mcp@3e907b6` clean; `rust-plex-mcp` hands-off (agy).
- **Deliberately not done:** No service drivers yet (Phase 2b+), no L2 `mcp` stdio (Phase 3), no L3 dashboard (Phase 4), no `CHANGELOG.md` (added Phase 2 closeout).
