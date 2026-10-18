# Development — `rust-arr-mcp`

## Toolchain

`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` first in every shell.
Stable Rust, Edition 2021. No nightly, no `cargo install` prerequisites beyond
`cargo` itself.

## Gates (all green before every commit)

```powershell
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo fmt --all --check
```

Deny list (workspace lints): `unwrap_used`, `expect_used`, `panic`, `todo`,
`unimplemented` — including tests. Approved patterns: `if let`, `let-else`,
`map`/`unwrap_or` (non-panicking), `saturating_*`, `.get()` indexing,
`ok()`-then-match in tests, explicit Demo clients in tests (never
`dispatch_auto` where counts matter — live config resolves on real machines).

## Recipes (`justfile`, copied from `rust-plex-mcp`)

`just` / `just check` / `just test` / `just fmt-check` / `just run-cli` /
`just run-mcp` / `just build-release` (plus `cargo clippy … -D warnings`,
`cargo fmt`, `cargo run -p arr-app -- --smoke` directly).

## Contributing

- One tier per commit, ≤10 files per commit, prove-by-running (paste gate output).
- Legacy `D:\Dev\repos\arr-mcp` is a read-only fence: verify SHA + clean status
  before and after every phase. `rust-plex-mcp` is hands-off (agy).
- Reconstructed wire shapes are flagged `pending live verify` in code comments
  and `docs/CONVERSION_LOG.md` — verify against a live service before relying.
- `HANDOVER.md` stays current (last action, next step, blockers, not-done).
