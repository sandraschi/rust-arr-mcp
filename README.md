# rust-arr-mcp

Pure-Rust *arr stack manager: 82-tool Model Context Protocol (MCP) server plus a native GPU-rendered desktop dashboard. One `cargo` toolchain, zero Python, zero Node.

## What this wraps

This project drives the ***arr family** — [Sonarr](https://sonarr.tv/) (TV), [Radarr](https://radarr.video/) (movies), [Lidarr](https://lidarr.audio/) (music), [Readarr](https://readarr.com/) (books), [Bazarr](https://www.bazarr.media/) (subtitles), [Prowlarr](https://prowlarr.com/) (indexers), [Overseerr](https://overseerr.dev/) (requests) — over their HTTP APIs (`X-Api-Key`, 30 s timeouts, 32 MiB caps). See [docs/WRAPPEE.md](docs/WRAPPEE.md) for project links and community resources.

Converted from [`arr-mcp`](https://github.com/sandraschi/arr-mcp) (`master@3e907b6`) under the fleet `repconv` standard (Phases 0-4, Shadow-Then-Swap). Conversion record: [docs/CONVERSION_DOSSIER.md](docs/CONVERSION_DOSSIER.md), [docs/CONVERSION_LOG.md](docs/CONVERSION_LOG.md), [CHANGELOG.md](CHANGELOG.md).

## What You Can Do

- **TV & Movies**: list/search/add/monitor Sonarr series and episodes, Radarr movies — with the same safety guards as the original (required IDs, mismatch blocks search, files never deleted by default).
- **Music & Books**: Lidarr artists/albums, Readarr authors/books plus book mutations and add-options lookup.
- **Subtitles & Indexers**: Bazarr wanted/search/download/history/providers/languages; Prowlarr indexers, unified search, application sync, history.
- **Requests**: Overseerr request lifecycle (create/approve/decline/delete/count/pending), media search, users.
- **Triple naming**: every tool answers to `sonarr.series_list`, `sonarr_series_list`, and legacy `sonarr_series(operation="list")` — existing prompts keep working with zero edits.
- **Desktop dashboard**: 19 native pages (7 services, health, chat, settings with vault-backed credentials, logs, raw JSON inspector).
- **AI Chat Copilot**: ask about your stack with a local Ollama or cloud model (Settings page).

## Install

### 1. Windows release (recommended)

1. Download **`rust-arr-mcp.exe`** and **`arr-app.exe`** from the [latest release](https://github.com/sandraschi/rust-arr-mcp/releases/latest).
2. Run `arr-app.exe` for the dashboard. Windows may show a SmartScreen warning for unsigned binaries: choose **More info > Run anyway**.

### 2. For Claude Desktop / Cursor / OpenCode (MCP server only)

Add `rust-arr-mcp` to your MCP client configuration (`claude_desktop_config.json`):

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

For full installation options, service setup (URLs + API keys), and source builds, see [INSTALL.md](INSTALL.md).
Developers: build instructions and test recipes are in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Example Prompts

Once configured in Claude or your AI agent, you can ask natural-language questions:

- *"What episodes are missing from my Sonarr library for Breaking Bad?"*
- *"Add Dune (TMDB 438631) to Radarr with quality profile 1 in /movies."*
- *"Which indexers are failing in Prowlarr?"*
- *"Show pending Overseerr requests."*

## Documentation

| Document | Description |
|---|---|
| [INSTALL.md](INSTALL.md) | All install paths (release, MCP-only, source) |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 3-crate layout, data flow, protocols |
| [docs/RUST.md](docs/RUST.md) | Rust toolchain, lints, gates, CI/CD, eframe notes |
| [docs/CONFIGURATION.md](docs/CONFIGURATION.md) | Env vars, vault, import order |
| [docs/TOOLS.md](docs/TOOLS.md) | Full 82-tool MCP reference |
| [docs/WRAPPEE.md](docs/WRAPPEE.md) | The *arr projects, links, communities |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Dev setup, gates, contributing |
| [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) | FAQ, errors, diagnostics |
| [CHANGELOG.md](CHANGELOG.md) | Release history |
