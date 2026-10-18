//! Zero-config legacy fleet import: auto-discover `../arr-mcp`, parse its
//! `.env` (with `${VAR:-default}` expansion) and `config/arr-services.json`.
//! Resolution order: explicit args > process env > OS vault > imported files.
//! Secrets are never logged.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ArrError;

/// Per-service connection settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ServiceConfig {
    /// Base URL, e.g. `http://localhost:8989`.
    pub url: Option<String>,
    /// API key (`X-Api-Key` header).
    pub api_key: Option<String>,
    /// Service enabled flag.
    pub enabled: bool,
}

/// All seven Arr services.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ArrServices {
    pub sonarr: ServiceConfig,
    pub radarr: ServiceConfig,
    pub lidarr: ServiceConfig,
    pub readarr: ServiceConfig,
    pub bazarr: ServiceConfig,
    pub prowlarr: ServiceConfig,
    pub overseerr: ServiceConfig,
}

impl ArrServices {
    /// Service names in dossier order.
    pub const NAMES: [&'static str; 7] = [
        "sonarr",
        "radarr",
        "lidarr",
        "readarr",
        "bazarr",
        "prowlarr",
        "overseerr",
    ];

    /// Get a service config by lowercase name.
    pub fn get(&self, name: &str) -> Option<&ServiceConfig> {
        match name {
            "sonarr" => Some(&self.sonarr),
            "radarr" => Some(&self.radarr),
            "lidarr" => Some(&self.lidarr),
            "readarr" => Some(&self.readarr),
            "bazarr" => Some(&self.bazarr),
            "prowlarr" => Some(&self.prowlarr),
            "overseerr" => Some(&self.overseerr),
            _ => None,
        }
    }
}

/// Locate the sibling legacy repo (`../arr-mcp` next to this workspace).
pub fn discover_legacy_dir() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sibling = manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("arr-mcp"))?;
    if sibling.join(".env").is_file() || sibling.join("config").is_dir() {
        Some(sibling)
    } else {
        None
    }
}

/// Parse dotenv text into a map. Skips blanks/`#` comments, strips optional
/// `export ` prefix and surrounding quotes. Never panics on malformed lines.
pub fn parse_dotenv(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let body = line.strip_prefix("export ").unwrap_or(line);
        let Some(eq) = body.find('=') else { continue };
        let key = body[..eq].trim();
        if key.is_empty() {
            continue;
        }
        let mut value = body[eq + 1..].trim();
        let double_quoted = value.len() >= 2 && value.starts_with('"') && value.ends_with('"');
        let single_quoted = value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'');
        if double_quoted || single_quoted {
            if let Some(stripped) = value.get(1..value.len().saturating_sub(1)) {
                value = stripped;
            }
        }
        map.insert(key.to_string(), expand_value(value, &map));
    }
    map
}

/// Expand `$VAR` and `${VAR:-default}` using process env first, then already
/// parsed dotenv entries. Unknown vars without defaults expand to empty.
pub fn expand_value(value: &str, dotenv: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'$' {
            if let Some(ch) = value[i..].chars().next() {
                out.push(ch);
                i += ch.len_utf8();
            } else {
                break;
            }
            continue;
        }
        // `${VAR:-default}` form.
        if bytes.get(i + 1) == Some(&b'{') {
            if let Some(end) = value[i..].find('}') {
                let inner = &value[i + 2..i + end];
                let (name, default) = match inner.find(":-") {
                    Some(pos) => (&inner[..pos], Some(&inner[pos + 2..])),
                    None => (inner, None),
                };
                let resolved = std::env::var(name)
                    .ok()
                    .or_else(|| dotenv.get(name).cloned())
                    .or_else(|| default.map(str::to_string))
                    .unwrap_or_default();
                out.push_str(&resolved);
                i += end + 1;
                continue;
            }
            out.push('$');
            i += 1;
            continue;
        }
        // `$VAR` form.
        let rest = &value[i + 1..];
        let len = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .map(char::len_utf8)
            .sum::<usize>();
        if len == 0 {
            out.push('$');
            i += 1;
            continue;
        }
        let name = &rest[..len];
        let resolved = std::env::var(name)
            .ok()
            .or_else(|| dotenv.get(name).cloned())
            .unwrap_or_default();
        out.push_str(&resolved);
        i += 1 + len;
    }
    out
}

/// Load `.env` + `config/arr-services.json` from a legacy checkout directory.
/// Missing files are tolerated (empty result); malformed JSON is a typed error.
pub fn load_from_dir(dir: &Path) -> Result<ArrServices, ArrError> {
    let mut services = ArrServices::default();

    let dotenv_path = dir.join(".env");
    let dotenv = std::fs::read_to_string(&dotenv_path)
        .ok()
        .map(|t| parse_dotenv(&t))
        .unwrap_or_default();

    // Uppercase `<SERVICE>_URL/_API_KEY/_ENABLED` convention from `.env.example`.
    apply_dotenv(&mut services, &dotenv);

    let services_json = dir.join("config").join("arr-services.json");
    if let Ok(text) = std::fs::read_to_string(&services_json) {
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| ArrError::Protocol(format!("arr-services.json parse error: {e}")))?;
        apply_services_json(&mut services, &parsed);
    }

    Ok(services)
}

fn apply_dotenv(services: &mut ArrServices, dotenv: &HashMap<String, String>) {
    let pairs = [
        ("sonarr", &mut services.sonarr),
        ("radarr", &mut services.radarr),
        ("lidarr", &mut services.lidarr),
        ("readarr", &mut services.readarr),
        ("bazarr", &mut services.bazarr),
        ("prowlarr", &mut services.prowlarr),
        ("overseerr", &mut services.overseerr),
    ];
    for (name, cfg) in pairs {
        let prefix = name.to_ascii_uppercase();
        if let Some(url) = dotenv.get(&format!("{prefix}_URL")) {
            if !url.is_empty() {
                cfg.url = Some(url.clone());
            }
        }
        if let Some(key) = dotenv.get(&format!("{prefix}_API_KEY")) {
            if !key.is_empty() && !key.starts_with("your-") {
                cfg.api_key = Some(key.clone());
            }
        }
        if let Some(enabled) = dotenv.get(&format!("{prefix}_ENABLED")) {
            cfg.enabled = matches!(enabled.to_ascii_lowercase().as_str(), "true" | "1" | "yes");
        }
    }
}

fn apply_services_json(services: &mut ArrServices, parsed: &serde_json::Value) {
    let targets = [
        ("sonarr", &mut services.sonarr),
        ("radarr", &mut services.radarr),
        ("lidarr", &mut services.lidarr),
        ("readarr", &mut services.readarr),
        ("bazarr", &mut services.bazarr),
        ("prowlarr", &mut services.prowlarr),
        ("overseerr", &mut services.overseerr),
    ];
    for (name, cfg) in targets {
        let node = parsed
            .get(name)
            .or_else(|| parsed.get("services").and_then(|s| s.get(name)));
        let Some(node) = node else { continue };
        if let Some(url) = node.get("url").and_then(|v| v.as_str()) {
            if !url.is_empty() {
                cfg.url = Some(url.to_string());
            }
        }
        if let Some(key) = node
            .get("api_key")
            .or_else(|| node.get("apiKey"))
            .and_then(|v| v.as_str())
        {
            if !key.is_empty() {
                cfg.api_key = Some(key.to_string());
            }
        }
        if let Some(enabled) = node.get("enabled").and_then(|v| v.as_bool()) {
            cfg.enabled = enabled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotenv_parses_pairs_skips_comments() {
        let map = parse_dotenv("# comment\n\nSONARR_URL=http://localhost:8989\nEMPTY=\nNOEQUALS\n");
        assert_eq!(
            map.get("SONARR_URL").map(String::as_str),
            Some("http://localhost:8989")
        );
        assert_eq!(map.get("EMPTY").map(String::as_str), Some(""));
        assert!(!map.contains_key("NOEQUALS"));
    }

    #[test]
    fn dotenv_expands_default_syntax() {
        let map = parse_dotenv("BASE=http://localhost\nFULL=${BASE:-http://x}:8989\n");
        assert_eq!(
            map.get("FULL").map(String::as_str),
            Some("http://localhost:8989")
        );
    }

    #[test]
    fn dotenv_ignores_placeholder_keys() {
        let map = parse_dotenv(
            "RADARR_API_KEY=your-radarr-api-key-here\nRADARR_URL=http://localhost:7878\n",
        );
        let mut services = ArrServices::default();
        apply_dotenv(&mut services, &map);
        assert!(services.radarr.api_key.is_none());
        assert_eq!(
            services.radarr.url.as_deref(),
            Some("http://localhost:7878")
        );
    }

    #[test]
    fn services_json_overrides_dotenv() {
        let map = parse_dotenv("SONARR_URL=http://localhost:8989\n");
        let mut services = ArrServices::default();
        apply_dotenv(&mut services, &map);
        let parsed: serde_json::Value =
            serde_json::from_str(r#"{"sonarr": {"url": "http://lan:8989", "enabled": true}}"#)
                .unwrap_or(serde_json::Value::Null);
        apply_services_json(&mut services, &parsed);
        assert_eq!(services.sonarr.url.as_deref(), Some("http://lan:8989"));
        assert!(services.sonarr.enabled);
    }
}
