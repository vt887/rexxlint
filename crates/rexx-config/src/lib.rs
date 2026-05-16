use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormattingProfile {
    pub name: String,
    pub line_length_soft: usize,
    pub line_length_hard: usize,
    pub uppercase_keywords: bool,
    pub tabs_forbidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RexxLintConfig {
    #[serde(default)]
    pub formatting: Option<FormattingProfile>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown formatting profile: {0}")]
    UnknownProfile(String),
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

pub fn mainframe_compatible() -> FormattingProfile {
    FormattingProfile {
        name: "mainframe-compatible".to_string(),
        line_length_soft: 72,
        line_length_hard: 80,
        uppercase_keywords: true,
        tabs_forbidden: true,
    }
}

pub fn standard() -> FormattingProfile {
    FormattingProfile {
        name: "standard".to_string(),
        line_length_soft: 100,
        line_length_hard: 200,
        uppercase_keywords: false,
        tabs_forbidden: false,
    }
}

pub fn minimal() -> FormattingProfile {
    FormattingProfile {
        name: "minimal".to_string(),
        line_length_soft: 200,
        line_length_hard: 200,
        uppercase_keywords: false,
        tabs_forbidden: false,
    }
}

impl Default for FormattingProfile {
    fn default() -> Self {
        mainframe_compatible()
    }
}

pub fn load_profile(name: &str) -> Result<FormattingProfile, ConfigError> {
    match name {
        "mainframe-compatible" | "mainframe" => Ok(mainframe_compatible()),
        "standard" => Ok(standard()),
        "minimal" => Ok(minimal()),
        _ => Err(ConfigError::UnknownProfile(name.to_string())),
    }
}

pub fn default_profile() -> FormattingProfile {
    mainframe_compatible()
}

pub fn find_config(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();
    loop {
        let config_path = current.join("rexxlint.toml");
        if config_path.exists() {
            return Some(config_path);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn load_config(path: &Path) -> Result<RexxLintConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: RexxLintConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn resolve_config(start_path: &Path) -> RexxLintConfig {
    if let Some(path) = find_config(start_path) {
        load_config(&path).unwrap_or_default()
    } else {
        RexxLintConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_config() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("a/b/c");
        fs::create_dir_all(&sub).unwrap();

        let config_path = dir.path().join("rexxlint.toml");
        fs::write(&config_path, "formatting.name = 'test'").unwrap();

        let found = find_config(&sub).unwrap();
        assert_eq!(
            found.canonicalize().unwrap(),
            config_path.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_load_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rexxlint.toml");
        fs::write(&config_path, "[formatting]\nname = 'custom'\nline_length_soft = 120\nline_length_hard = 140\nuppercase_keywords = true\ntabs_forbidden = false").unwrap();

        let config = load_config(&config_path).unwrap();
        let fmt = config.formatting.unwrap();
        assert_eq!(fmt.name, "custom");
        assert_eq!(fmt.line_length_soft, 120);
        assert_eq!(fmt.line_length_hard, 140);
        assert!(fmt.uppercase_keywords);
        assert!(!fmt.tabs_forbidden);
    }
}
