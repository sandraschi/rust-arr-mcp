# Rust — `rust-arr-mcp` toolchain, lints, workspace, gates, CI/CD

Fleet note first: **ruff, biome, pytest, and tsc do not apply here.** Those are
the Python/FastMCP + React track. The Rust track runs on four gates —
`clippy`, `test`, `fmt`, `smoke` — locally via `cargo`/`just` and in GitHub
Actions via `.github/workflows/ci.yml` (Windows-only, stable toolchain).

## Toolchain

- Stable Rust via `rustup` (`rustup update stable`). Edition **2021**
  (deliberate: `let`-chains and other Edition 2024 features do **not** compile —
  caught live in Phase 2a, fixed by expanding to nested `if let`).
- PATH first in every shell: `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`
  (all `just` recipes do this so NSSM/IDE shells work without inherited PATH).
- MSRV reality: `eframe`/`egui 0.37` sometimes need newer `cargo`/rustc for
  dependency resolution — `rustup update` is the first fix for mystery build
  failures, not feature flags.

## Workspace (3 crates, explicit members)

```toml
[workspace]
members = ["crates/arr-core", "crates/arr-mcp", "crates/arr-app"]
resolver = "2"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
unimplemented = "deny"
todo = "deny"
```

Shared deps live in `[workspace.dependencies]` (`serde`, `reqwest`, `tokio`
with `rt-multi-thread/macros/time` only — never `full`, `chrono`, `uuid`,
`rusqlite/bundled`, `keyring`, `serde_yaml`, `eframe 0.37`, `egui_kittest`).
`Cargo.lock` is **committed** (reproducible CI + releases).

## The deny gate (what it forces)

| Forbidden | Approved instead |
|---|---|
| `unwrap()` / `expect()` (incl. tests) | `if let`, `let-else`, `map`, `ok()`-then-match |
| `panic!` / `todo!` / `unimplemented!` | typed `ArrError` + `?` bubbling |
| `buf[i]` indexing | `.get()` |
| `s[i..j]` string slicing | char-boundary iteration |
| bare `+ - *` on untrusted ints | `checked_*` / `saturating_*` |
| `#[allow]` to silence a gate | fix the code (test-only `allow` needs a comment) |

## Gates (run in this order)

```powershell
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo fmt --all --check
cargo run -p arr-app -- --smoke   # headless, CI-safe, no window
```

`--all-targets` matters: it lints test code too (an `expect()` in a test fails
the gate — caught live in Phase 2a). `smoke` asserts status + list shapes
without opening a window.

## CI/CD (GitHub Actions, Windows-only)

`.github/workflows/ci.yml`: push/PR on `main` (+ manual dispatch) →
checkout → `dtolnay/rust-toolchain@stable` with clippy+rustfmt → the four
gates + smoke. No release build in CI (eframe release ≈ 2 min; it rides
`just build-release` + GitHub releases with attached exes). Private repos keep
Actions account-disabled as usual — `just ci` runs the identical sequence
locally.

## eframe 0.37 API notes (paid for in Phase 4)

- `impl App` uses `fn ui(&mut self, ui: &mut Ui, frame)` — `fn update(ctx)`
  does not exist; `SidePanel`/`TopBottomPanel`/`CentralPanel(ctx)` are gone,
  use `Panel::top/left/bottom` + `CentralPanel` with `&mut Ui`.
- `egui::Frame::NONE` (not `::none()`), `ctx` comes from `ui.ctx().clone()`,
  repaint while jobs are pending or the UI stalls.
- `include_bytes!` fails the build when the file is missing — no icon asset
  means no icon block (don't reference `icons/icon.png` until it exists).

## Release flow

```powershell
just build-release   # target\release\rust-arr-mcp.exe (~5 MB) + arr-app.exe (~20 MB)
```

Attach both exes to the GitHub release (`v0.1.0` pattern). No NSIS wrapper yet
(tracked in `HANDOVER.md`). `CHANGELOG.md` gets a section per release; the
`[Unreleased]` convention is skipped — checkpoint commits carry the narrative
until tagging day.
