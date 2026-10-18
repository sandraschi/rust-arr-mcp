//! L1 arr-core: *arr REST protocols, state machine, command bus.
//! Zero UI, zero MCP dependencies. Drivers: `sonarr` (Phase 2b); radarr,
//! lidarr, readarr, bazarr, prowlarr, overseerr land in later Phase 2 tiers.

pub mod command;
pub mod error;
pub mod import;
pub mod sonarr;

pub use command::{dispatch, dispatch_auto, Command};
pub use error::ArrError;
