//! L1 arr-core: *arr REST protocols, state machine, command bus.
//! Zero UI, zero MCP dependencies. Drivers: `sonarr` (Phase 2b), `radarr`
//! (Phase 2c), `lidarr` + `readarr` (Phase 2d); bazarr, prowlarr, overseerr
//! land in Phase 2e. Cross/health compose in Phase 3.

pub mod bazarr;
pub mod command;
pub mod error;
pub mod import;
pub mod lidarr;
pub mod overseerr;
pub mod prowlarr;
pub mod radarr;
pub mod readarr;
pub mod sonarr;

pub use command::{
    dispatch, dispatch_auto, dispatch_bazarr, dispatch_lidarr, dispatch_overseerr,
    dispatch_prowlarr, dispatch_radarr, dispatch_readarr, Command,
};
pub use error::ArrError;
