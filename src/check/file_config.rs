//! Configuration file parser for `.scopelint` file.
//!
//! Supports:
//! - File-level ignores (entire files)
//! - Rule-specific ignores per file (overrides)
//!
//! Format:
//! ```toml
//! # Ignore entire files
//! [ignore]
//! files = [
//!     "src/legacy/old.sol",
//!     "test/integration/*.sol"
//! ]
//!
//! # Ignore specific rules for specific files
//! [ignore.overrides]
//! "src/BaseBridgeReceiver.sol" = ["src"]
//! "src/legacy/**/*.sol" = ["src", "error"]
//! ```

use crate::check::utils::ValidatorKind;
use globset::{Glob, GlobMatcher};
use std::path::{Path, PathBuf};

/// Configuration loaded from `.scopelint` file
#[derive(Debug, Default, Clone)]
pub struct FileConfig {
    /// Directory where the `.scopelint` file was found (project root)
    config_dir: Option<PathBuf>,
    /// Patterns for files to ignore entirely
    ignored_file_patterns: Vec<GlobMatcher>,
    /// Rule-specific overrides: file pattern -> list of rules to ignore
    rule_overrides: Vec<(GlobMatcher, Vec<ValidatorKind>)>,
}

impl FileConfig {
    /// Load configuration from `.scopelint` file.
    /// Searches up the directory tree from the current working directory to find the file.
    /// Returns default config if file doesn't exist or can't be parsed.
    #[must_use]
    pub fn load() -> Self {
        let config_path = Self::find_config_file();
        let Some(config_path) = config_path else {
            return Self::default();
        };

        let config_dir = config_path.parent().map(PathBuf::from);

        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                let mut config = Self::from_toml(&content).unwrap_or_else(|err| {
                    eprintln!("Warning: Failed to parse .scopelint: {err}. Using default config.");
                    Self::default()
                });
                config.config_dir = config_dir;
                config
            }
            Err(err) => {
                eprintln!("Warning: Failed to read .scopelint: {err}. Using default config.");
                Self::default()
            }
        }
    }

    /// Search up the directory tree to find `.scopelint` file.
    /// Returns the path to the config file if found, None otherwise.
    fn find_config_file() -> Option<PathBuf> {
        let mut current_dir = std::env::current_dir().ok()?;

        loop {
            let config_path = current_dir.join(".scopelint");
            if config_path.exists() && config_path.is_file() {
                return Some(config_path);
            }

            // Move up one directory
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break, // Reached filesystem root
            }
        }

        None
    }

    /// Parse configuration from TOML string
    fn from_toml(content: &str) -> Result<Self, String> {
        let toml: toml::Value =
            toml::from_str(content).map_err(|e| format!("Invalid TOML: {e}"))?;

        let mut config = Self::default();

        // Parse [ignore] section
        if let Some(ignore_section) = toml.get("ignore") {
            // Parse files array
            if let Some(files) = ignore_section.get("files").and_then(|v| v.as_array()) {
                for file_pattern in files {
                    if let Some(pattern_str) = file_pattern.as_str() {
                        let glob = Glob::new(pattern_str)
                            .map_err(|e| format!("Invalid glob pattern '{pattern_str}': {e}"))?;
                        config.ignored_file_patterns.push(glob.compile_matcher());
                    }
                }
            }

            // Parse [ignore.overrides] section
            if let Some(overrides) = ignore_section.get("overrides").and_then(|v| v.as_table()) {
                for (pattern_str, rules_value) in overrides {
                    let glob = Glob::new(pattern_str)
                        .map_err(|e| format!("Invalid glob pattern '{pattern_str}': {e}"))?;
                    let matcher = glob.compile_matcher();

                    // Parse rules array
                    let rules = rules_value
                        .as_array()
                        .ok_or_else(|| format!("Rules for '{pattern_str}' must be an array"))?;

                    let mut validator_kinds = Vec::new();
                    for rule_str in rules {
                        let rule_name = rule_str
                            .as_str()
                            .ok_or_else(|| "Rule names must be strings".to_string())?;
                        let kind = parse_rule_name(rule_name)
                            .ok_or_else(|| format!("Unknown rule: '{rule_name}'"))?;
                        validator_kinds.push(kind);
                    }

                    config.rule_overrides.push((matcher, validator_kinds));
                }
            }
        }

        Ok(config)
    }

    /// Check if a file should be ignored entirely
    #[must_use]
    pub fn is_file_ignored(&self, file_path: &Path) -> bool {
        let normalized = self.normalize_path(file_path);

        self.ignored_file_patterns.iter().any(|matcher| matcher.is_match(&normalized))
    }

    /// Get list of rules to ignore for a specific file
    #[must_use]
    pub fn get_ignored_rules(&self, file_path: &Path) -> Vec<ValidatorKind> {
        let normalized = self.normalize_path(file_path);

        let mut ignored_rules = Vec::new();
        for (matcher, rules) in &self.rule_overrides {
            if matcher.is_match(&normalized) {
                ignored_rules.extend(rules.iter().cloned());
            }
        }
        ignored_rules
    }

    /// Normalize file path for glob matching:
    /// - Convert to relative path from config directory (project root)
    /// - Normalize path separators to forward slashes
    fn normalize_path(&self, file_path: &Path) -> String {
        // Use config directory as base, fallback to current directory if no config found
        let base_dir = self.config_dir.as_ref().map_or_else(
            || std::env::current_dir().ok().unwrap_or_else(|| PathBuf::from(".")),
            Clone::clone,
        );

        // Try to get relative path from base directory
        let relative = if file_path.is_absolute() {
            file_path.strip_prefix(&base_dir).unwrap_or(file_path)
        } else {
            file_path
        };

        let file_str = relative.to_string_lossy();
        // Normalize path separators for glob matching (Windows uses backslashes)
        let normalized = file_str.replace('\\', "/");
        // Strip leading "./" if present, as glob patterns don't expect it
        if normalized.starts_with("./") {
            normalized.strip_prefix("./").unwrap_or(&normalized).to_string()
        } else {
            normalized
        }
    }
}

/// Maps a rule name (e.g., "error") to a `ValidatorKind`
fn parse_rule_name(rule: &str) -> Option<ValidatorKind> {
    match rule {
        "error" => Some(ValidatorKind::Error),
        "import" => Some(ValidatorKind::Import),
        "variable" => Some(ValidatorKind::Variable),
        "constant" => Some(ValidatorKind::Constant),
        "test" => Some(ValidatorKind::Test),
        "script" => Some(ValidatorKind::Script),
        "src" => Some(ValidatorKind::Src),
        "eip712" => Some(ValidatorKind::Eip712),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ignore() {
        let toml = r#"
[ignore]
files = ["src/legacy.sol", "test/integration/*.sol"]
"#;
        let config = FileConfig::from_toml(toml).unwrap();

        assert!(config.is_file_ignored(Path::new("src/legacy.sol")));
        assert!(config.is_file_ignored(Path::new("test/integration/test.sol")));
        assert!(!config.is_file_ignored(Path::new("src/normal.sol")));
    }

    #[test]
    fn test_parse_rule_overrides() {
        let toml = r#"
[ignore.overrides]
"src/BaseBridgeReceiver.sol" = ["src"]
"src/legacy/**/*.sol" = ["src", "error"]
"#;
        let mut config = FileConfig::from_toml(toml).unwrap();
        // Set config_dir to simulate real scenario
        config.config_dir = Some(PathBuf::from("."));

        let ignored = config.get_ignored_rules(Path::new("src/BaseBridgeReceiver.sol"));
        assert_eq!(ignored, vec![ValidatorKind::Src]);

        let ignored = config.get_ignored_rules(Path::new("src/legacy/old.sol"));
        assert_eq!(ignored.len(), 2);
        assert!(ignored.contains(&ValidatorKind::Src));
        assert!(ignored.contains(&ValidatorKind::Error));
    }

    #[test]
    fn test_parse_empty_config() {
        let config = FileConfig::from_toml("").unwrap();
        assert!(!config.is_file_ignored(Path::new("src/test.sol")));
        assert!(config.get_ignored_rules(Path::new("src/test.sol")).is_empty());
    }
}
