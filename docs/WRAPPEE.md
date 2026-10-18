# Wrappee — the *arr stack

`rust-arr-mcp` drives seven independent open-source projects (the *arr / Servarr family):

| Service | What | Default port | Links |
|---|---|---|---|
| [Sonarr](https://sonarr.tv/) | TV series PVR | `:8989` | [Docs](https://wiki.servarr.com/sonarr) · [GitHub](https://github.com/Sonarr/Sonarr) · [Discord](https://discord.gg/sonarr) · [Reddit](https://www.reddit.com/r/sonarr/) |
| [Radarr](https://radarr.video/) | Movie collection manager | `:7878` | [Docs](https://wiki.servarr.com/radarr) · [GitHub](https://github.com/Radarr/Radarr) · [Reddit](https://www.reddit.com/r/radarr/) |
| [Lidarr](https://lidarr.audio/) | Music collection manager | `:8686` | [Docs](https://wiki.servarr.com/lidarr) · [GitHub](https://github.com/Lidarr/Lidarr) |
| [Readarr](https://readarr.com/) | Ebook/audiobook manager | `:8787` | [Docs](https://wiki.servarr.com/readarr) · [GitHub](https://github.com/Readarr/Readarr) |
| [Bazarr](https://www.bazarr.media/) | Subtitle companion | `:6767` | [Docs](https://docs.bazarr.media/) · [GitHub](https://github.com/morpheus65535/bazarr) |
| [Prowlarr](https://prowlarr.com/) | Indexer manager/proxy | `:9696` | [Docs](https://wiki.servarr.com/prowlarr) · [GitHub](https://github.com/Prowlarr/Prowlarr) |
| [Overseerr](https://overseerr.dev/) | Request management | `:5055` | [Docs](https://docs.overseerr.dev/) · [GitHub](https://github.com/sct/overseerr) · [Discord](https://discord.gg/overseerr) |

All Servarr apps share one auth model (`X-Api-Key` header, per-service key from
Settings → General) and one API style (`/api/v3` Sonarr/Radarr, `/api/v1` the
rest; Bazarr `/api`; Overseerr absolute `/api/v1/*`). Any subset may be
configured — the rest serve Demo fixtures and say so.
