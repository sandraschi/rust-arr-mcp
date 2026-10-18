# HANDOVER — `rust-arr-mcp`

- **Being worked on:** Phase 0 dossier gate (repconv `arr-mcp` → `rust-arr-mcp`, Shadow-Then-Swap).
- **Last concrete action (2026-10-18):** Recon complete + `docs/CONVERSION_DOSSIER.md`/`CONVERSION_LOG.md`/`.gitignore` written, not yet committed. No `.rs` written (gate holds).
- **Next step:** `git init` + `git add .gitignore docs/ HANDOVER.md` + commit `docs(phase-0): conversion dossier and pre-flight audit`. Then Phase 1 stamp (workspace `Cargo.toml` + kernel copy + `cargo check`, operation literals via live legacy `tools/list`).
- **Blockers:** None. Legacy `arr-mcp@3e907b6` clean; `rust-plex-mcp` explicitly hands-off (agy).
- **Deliberately not done:** No `Cargo.toml`/crates yet (Phase 1), no L1 code (Phase 2), no operation-literal lock (Phase 1 appends to dossier).
