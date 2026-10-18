# Configuration — `rust-arr-mcp`

## Resolution order (applies to every service)

1. **Explicit args**: CLI `--base-url` / `--api-key` flags.
2. **Process env**: `<SERVICE>_URL` + `<SERVICE>_API_KEY` (`SONARR_URL`, `RADARR_API_KEY`, …).
3. **OS vault**: Windows Credential Manager under service `rust-arr-mcp` (slots `<service>_url`, `<service>_key`, `llm_api_key`) — written from the dashboard Settings page, never plain text.
4. **Imported legacy checkout**: sibling `../arr-mcp` `.env` (`<SERVICE>_URL/_API_KEY/_ENABLED`, `your-*` placeholders rejected) + `config/arr-services.json` (overrides dotenv).
5. **Demo fixtures**: deterministic offline data, clearly Demo (unconfigured machines never crash, never hang).

## Variables

| Variable | Service | Example |
|---|---|---|
| `SONARR_URL` / `SONARR_API_KEY` | Sonarr | `http://localhost:8989` |
| `RADARR_URL` / `RADARR_API_KEY` | Radarr | `http://localhost:7878` |
| `LIDARR_URL` / `LIDARR_API_KEY` | Lidarr | `http://localhost:8686` |
| `READARR_URL` / `READARR_API_KEY` | Readarr | `http://localhost:8787` |
| `BAZARR_URL` / `BAZARR_API_KEY` | Bazarr | `http://localhost:6767` |
| `PROWLARR_URL` / `PROWLARR_API_KEY` | Prowlarr | `http://localhost:9696` |
| `OVERSEERR_URL` / `OVERSEERR_API_KEY` | Overseerr | `http://localhost:5055` |

Any subset may be set. Overseerr is commonly disabled (leave unset for Demo).

## App state (L3)

- Vault: OS keyring (see above). Pinned service profiles (name + URL, no secrets): `%LOCALAPPDATA%\rust-arr-mcp\services.json` (corrupt file → empty registry, never a crash).
- Crash log: `%LOCALAPPDATA%\rust-arr-mcp\crash.log` (forensic panic hook).
