//! L1 arr-core: *arr REST protocols, state machine, command bus.
//! Zero UI, zero MCP dependencies. Drivers: `sonarr` (Phase 2b), `radarr`
//! (Phase 2c); lidarr, readarr, bazarr, prowlarr, overseerr land in later tiers.

pub mod command;
pub mod error;
pub mod import;
pub mod radarr;
pub mod sonarr;

pub use command::{dispatch, dispatch_auto, dispatch_radarr, Command};
pub use error::ArrError;
