# Troubleshooting — `rust-arr-mcp`

## `auth` / 401-403 errors

The API key is wrong or the wrong service's key. Re-paste from the service UI
(Settings → General → API Key) into dashboard Settings → Save to vault, or set
`<SERVICE>_API_KEY` in the environment. Keys are never logged — check presence,
never values.

## `Offline` / unreachable

The service isn't running or the URL is wrong (wrong port is the classic:
Sonarr `:8989`, Radarr `:7878`, Lidarr `:8686`, Readarr `:8787`, Bazarr `:6767`,
Prowlarr `:9696`, Overseerr `:5055`). `health --probe` shows per-service
liveness. Unconfigured services serve Demo fixtures by design — if you see Demo
data, you simply haven't configured that service yet.

## Timeouts on big libraries

Reads cap at 32 MiB / 30 s. A `series_list` on a 10k-item library can approach
the cap — narrow with lookup terms instead.

## MCP client shows no tools / handshake fails

1. Confirm the binary runs: `rust-arr-mcp.exe status` prints JSON.
2. Replay the probe from `INSTALL.md` (initialize → expect `protocolVersion`).
3. Stdout must be pure JSON-RPC: our logs go to stderr. If a wrapper merges the
   streams, split them — that breaks every MCP server, not just this one.
4. String and numeric request IDs both work (echoed verbatim).

## Dashboard shows Demo data

Expected until configured (see above). Per-service: Settings → pick service →
paste URL + key → Save to vault → page Refresh.

## Crash on startup

Read `%LOCALAPPDATA%\rust-arr-mcp\crash.log` (forensic hook writes the panic +
timestamp). Paste it with your report — it names the exact site.

## Search was not started (mismatch guard)

`series_search`/`movie_search`/etc. verify the ID round-trips before firing the
native `*Search` command. A mismatch error means the service returned a different
record than requested — re-list and re-check the ID; nothing was submitted.
