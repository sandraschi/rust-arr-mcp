# Installing `rust-arr-mcp` & Arr ArtCraft Desktop

## Prerequisites

- **Windows 10/11 x64** (primary), macOS/Linux build from source
- **One or more *arr services** running and reachable: Sonarr `:8989`, Radarr `:7878`, Lidarr `:8686`, Readarr `:8787`, Bazarr `:6767`, Prowlarr `:9696`, Overseerr `:5055` (any subset works — unconfigured services fall back to Demo fixtures, never crash)
- Each service's **API key** (Settings → General in each *arr UI)
- **Rust stable** via `rustup` (source builds only; Edition 2021, no nightly features)

## Option A — Release binaries (recommended)

1. Download **`rust-arr-mcp.exe`** (CLI + MCP server) and **`arr-app.exe`** (dashboard) from the [latest release](https://github.com/sandraschi/rust-arr-mcp/releases/latest).
2. Run `arr-app.exe`. On first launch open **Settings**, pick each service, paste URL + API key, **Save to vault** (Windows Credential Manager — never plain text).
3. SmartScreen warning for unsigned binaries: **More info > Run anyway**.

## Option B — Claude Desktop / Cursor / OpenCode (MCP server only)

```json
{
  "mcpServers": {
    "arr": {
      "command": "C:\\path\\to\\rust-arr-mcp.exe",
      "args": ["mcp"]
    }
  }
}
```

Credentials resolve automatically: CLI flags > process env (`SONARR_URL`/`SONARR_API_KEY`, `RADARR_*`, `LIDARR_*`, `READARR_*`, `BAZARR_*`, `PROWLARR_*`, `OVERSEERR_*`) > OS vault > sibling `arr-mcp/.env` + `config/arr-services.json` > Demo fixtures. See [CONFIGURATION.md](docs/CONFIGURATION.md).

## Option C — Build from source

```powershell
# Rust toolchain on PATH
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

cd rust-arr-mcp

# Debug build + gates
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo fmt --all --check

# Release binaries (L2 CLI/MCP + L3 egui desktop app)
cargo build --release -p rust-arr-mcp -p arr-app
# → target\release\rust-arr-mcp.exe, target\release\arr-app.exe

# Headless smoke (no window, CI-safe)
.\target\debug\arr-app.exe --smoke
```

## Verify the install

```powershell
# CLI status (no network)
.\rust-arr-mcp.exe status

# One live read per configured service
.\rust-arr-mcp.exe health --probe

# MCP handshake (expect protocolVersion + 82 tools)
'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}' | .\rust-arr-mcp.exe mcp
```
