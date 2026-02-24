//! Path configuration from `foundry.toml`.
//!
//! Reads the existing Foundry config so scopelint works with non-default layouts
//! (e.g. `contracts/` instead of `src/`). Paths can be overridden with a
//! scopelint-specific `[check]` section.

use std::path::PathBuf;

/// Paths for source, script, and test directories (relative to project root).
/// Normalized to start with `./` for consistent use with walking and path checks.
#[derive(Debug, Clone)]
pub struct CheckPaths {
    /// Source contracts directory (e.g. `./src` or `./contracts`).
    pub src_path: String,
    /// Scripts directory (e.g. `./script`).
    pub script_path: String,
    /// Test directory (e.g. `./test`).
    pub test_path: String,
}

impl Default for CheckPaths {
    fn default() -> Self {
        Self {
            src_path: "./src".to_string(),
            script_path: "./script".to_string(),
            test_path: "./test".to_string(),
        }
    }
}

impl CheckPaths {
    /// Paths as a 3-element array for iterating (src, script, test).
    #[must_use]
    pub const fn as_array(&self) -> [&str; 3] {
        [self.src_path.as_str(), self.script_path.as_str(), self.test_path.as_str()]
    }

    /// Load paths from `foundry.toml`: use `[check]` overrides if present,
    /// otherwise `[profile.default]` (or root-level) `src`, `test`, `script`.
    /// Returns default paths if no config is found or parsing fails.
    #[must_use]
    pub fn load() -> Self {
        let Some(config_path) = Self::find_foundry_toml() else {
            return Self::default();
        };

        let Ok(content) = std::fs::read_to_string(&config_path) else {
            return Self::default();
        };

        Self::from_toml(&content).unwrap_or_default()
    }

    fn find_foundry_toml() -> Option<PathBuf> {
        let mut current_dir = std::env::current_dir().ok()?;

        loop {
            let config_path = current_dir.join("foundry.toml");
            if config_path.exists() && config_path.is_file() {
                return Some(config_path);
            }

            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break,
            }
        }

        None
    }

    /// Parse paths from TOML. Uses `[check]` section if present, else Foundry's
    /// `[profile.default]` (or root) `src`, `test`, `script`.
    pub(crate) fn from_toml(content: &str) -> Result<Self, String> {
        let toml: toml::Value =
            toml::from_str(content).map_err(|e| format!("Invalid TOML: {e}"))?;

        // Optional scopelint [check] overrides (src_path, script_path, test_path)
        let check_section = toml.get("check").and_then(|v| v.as_table());

        let (src_path, script_path, test_path) = check_section.map_or_else(
            || {
                (
                    from_foundry_profile(&toml, "src"),
                    from_foundry_profile(&toml, "script"),
                    from_foundry_profile(&toml, "test"),
                )
            },
            |check| {
                let src = check.get("src_path").and_then(|v| v.as_str()).map(normalize_path);
                let script = check.get("script_path").and_then(|v| v.as_str()).map(normalize_path);
                let test = check.get("test_path").and_then(|v| v.as_str()).map(normalize_path);
                (
                    src.unwrap_or_else(|| from_foundry_profile(&toml, "src")),
                    script.unwrap_or_else(|| from_foundry_profile(&toml, "script")),
                    test.unwrap_or_else(|| from_foundry_profile(&toml, "test")),
                )
            },
        );

        Ok(Self { src_path, script_path, test_path })
    }
}

/// Read a path from [profile.default] or root level (Foundry allows both).
fn from_foundry_profile(toml: &toml::Value, key: &str) -> String {
    let profile = toml
        .get("profile")
        .and_then(|p| p.get("default"))
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_str());
    let root = toml.get(key).and_then(|v| v.as_str());
    let raw = profile.or(root).unwrap_or(match key {
        "script" => "script",
        "test" => "test",
        _ => "src",
    });
    normalize_path(raw)
}

/// Ensure path has a `./` prefix for consistent comparison and walking.
fn normalize_path(p: &str) -> String {
    let trimmed = p.trim();
    if trimmed.is_empty() {
        return "./.".to_string();
    }
    if trimmed.starts_with("./") || trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("./{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::CheckPaths;

    #[test]
    fn from_toml_defaults_when_no_paths() {
        // No src/test/script in config -> use Foundry defaults
        let p = CheckPaths::from_toml("[fmt]\nline_length = 100").unwrap();
        assert_eq!(p.src_path, "./src");
        assert_eq!(p.script_path, "./script");
        assert_eq!(p.test_path, "./test");
    }

    #[test]
    fn from_toml_profile_default() {
        let p = CheckPaths::from_toml(
            r#"
[profile.default]
src = "contracts"
test = "test"
script = "script"
"#,
        )
        .unwrap();
        assert_eq!(p.src_path, "./contracts");
        assert_eq!(p.script_path, "./script");
        assert_eq!(p.test_path, "./test");
    }

    #[test]
    fn from_toml_check_overrides() {
        let p = CheckPaths::from_toml(
            r#"
[profile.default]
src = "src"
test = "test"
script = "script"

[check]
src_path = "./contracts"
script_path = "./scripts"
test_path = "./tests"
"#,
        )
        .unwrap();
        assert_eq!(p.src_path, "./contracts");
        assert_eq!(p.script_path, "./scripts");
        assert_eq!(p.test_path, "./tests");
    }

    #[test]
    fn from_toml_check_partial_override_falls_back_to_profile() {
        let p = CheckPaths::from_toml(
            r#"
[profile.default]
src = "contracts"
test = "test"
script = "script"

[check]
src_path = "./contracts"
"#,
        )
        .unwrap();
        assert_eq!(p.src_path, "./contracts");
        assert_eq!(p.script_path, "./script");
        assert_eq!(p.test_path, "./test");
    }
}
