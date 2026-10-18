set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

# rust-arr-mcp automation (ArtCraft cadcraft pattern).
# Every recipe prefixes cargo via USERPROFILE so NSSM / IDE shells work
# even when .cargo/bin is not on the inherited PATH.
# Rust track: clippy + test + fmt + smoke. No ruff / biome / pytest here.

default:
    @just --list

# Full local gate (mirrors .github/workflows/ci.yml)
ci: clippy test fmt-check smoke

# Strict ArtCraft lint gate across all workspace members
check:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings

clippy: check

# Unit + integration tests
test:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo test --workspace

# Formatting gate
fmt-check:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo fmt --all --check

# Format in place
fmt:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo fmt --all

# Headless L3 smoke (no window, CI-safe)
smoke:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo run -p arr-app -- --smoke

# Start the MCP server locally over stdio (logs on stderr, JSON-RPC on stdout)
run-mcp:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; $env:RUST_LOG = "debug"; cargo run -p rust-arr-mcp -- mcp

# CLI smoke: status without MCP
run-cli:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo run -p rust-arr-mcp -- status

# Production build (GUI + MCP binaries)
build-release:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo build --release --workspace

# Hourly maintenance: lint, test, changelog checkpoint (commit + push done by operator)
sync-hourly:
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings
    $env:Path = "$env:USERPROFILE\\.cargo\\bin;$env:Path"; cargo test --workspace
    Add-Content -Path "{{justfile_directory()}}\\CHANGELOG.md" -Value "- Checkpoint: $(Get-Date -Format 'yyyy-MM-dd HH:mm') UTC (clippy + tests green)"
