//! L1 arr-core: *arr REST protocols, state machine, command bus.
//! Zero UI, zero MCP dependencies. Domain drivers land in Phase 2.

pub mod error;
pub mod import;

use serde_json::Value;

use crate::error::ArrError;

/// Universal command bus (Phase 1 seed; domain variants land in Phase 2).
pub enum Command {
    Status,
}

/// Dispatch a command to a JSON value. Never panics; all failures are typed.
pub fn dispatch(cmd: Command) -> Result<Value, ArrError> {
    match cmd {
        Command::Status => Ok(serde_json::json!({
            "service": "rust-arr-mcp",
            "phase": "phase-1-stamp",
            "ok": true,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_future_is_send() {
        fn assert_send<T: Send>(_: T) {}
        assert_send(dispatch(Command::Status));
    }

    #[test]
    fn status_round_trip() {
        let v = dispatch(Command::Status);
        assert!(v.is_ok());
        if let Ok(v) = v {
            assert_eq!(v.get("ok"), Some(&serde_json::Value::Bool(true)));
        }
    }
}
