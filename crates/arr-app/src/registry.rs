//! Pinned Arr service profiles: user-pinned service name + URL pairs.
//!
//! Secrets stay in the OS vault (`vault.rs`); this registry holds only names
//! and URLs as plain JSON next to the app state. Corrupt file -> empty
//! registry, never a crash.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegService {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    pub services: Vec<RegService>,
}

pub fn path() -> Option<std::path::PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(|base| {
        std::path::PathBuf::from(base)
            .join("rust-arr-mcp")
            .join("services.json")
    })
}

pub fn load() -> Registry {
    let Some(path) = path() else {
        return Registry::default();
    };
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(registry: &Registry) {
    let Some(path) = path() else { return };
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    if let Ok(text) = serde_json::to_string_pretty(registry) {
        let _ = std::fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_file_is_empty_registry() {
        let registry: Registry = serde_json::from_str("not json{{").unwrap_or_default();
        assert!(registry.services.is_empty());
    }

    #[test]
    fn registry_manipulations() {
        let mut reg = Registry::default();
        reg.services.push(RegService {
            name: "sonarr".to_string(),
            url: "http://localhost:8989".to_string(),
        });
        assert_eq!(reg.services.len(), 1);
        reg.services.retain(|l| l.name != "sonarr");
        assert!(reg.services.is_empty());
    }
}
