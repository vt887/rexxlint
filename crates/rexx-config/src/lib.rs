use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeywordCasing {
    #[default]
    Upper,
    Lower,
    Preserve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormattingProfile {
    pub name: String,
    pub line_length_soft: usize,
    pub line_length_hard: usize,
    pub tabs_forbidden: bool,
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_max_blank_lines")]
    pub max_blank_lines: usize,
    #[serde(default = "default_true")]
    pub insert_first_comment: bool,
    #[serde(default)]
    pub keyword_casing: KeywordCasing,
}

fn default_true() -> bool {
    true
}
fn default_indent_size() -> usize {
    4
}
fn default_max_blank_lines() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesConfig {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RexxLintConfig {
    #[serde(default)]
    pub formatting: Option<FormattingProfile>,
    #[serde(default)]
    pub files: Option<FilesConfig>,
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
        tabs_forbidden: true,
        indent_size: 4,
        max_blank_lines: 1,
        insert_first_comment: true,
        keyword_casing: KeywordCasing::Upper,
    }
}

pub fn standard() -> FormattingProfile {
    FormattingProfile {
        name: "standard".to_string(),
        line_length_soft: 100,
        line_length_hard: 200,
        tabs_forbidden: false,
        indent_size: 4,
        max_blank_lines: 1,
        insert_first_comment: true,
        keyword_casing: KeywordCasing::Lower,
    }
}

pub fn minimal() -> FormattingProfile {
    FormattingProfile {
        name: "minimal".to_string(),
        line_length_soft: 200,
        line_length_hard: 200,
        tabs_forbidden: false,
        indent_size: 4,
        max_blank_lines: 2,
        insert_first_comment: false,
        keyword_casing: KeywordCasing::Preserve,
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
        assert_eq!(found, config_path);
    }

    #[test]
    fn test_load_profile_mainframe() {
        let p = load_profile("mainframe-compatible").unwrap();
        assert_eq!(p.keyword_casing, KeywordCasing::Upper);
        assert!(p.tabs_forbidden);
        assert_eq!(p.indent_size, 4);
        assert_eq!(p.max_blank_lines, 1);
        assert!(p.insert_first_comment);
    }

    #[test]
    fn test_load_profile_standard() {
        let p = load_profile("standard").unwrap();
        assert_eq!(p.keyword_casing, KeywordCasing::Lower);
        assert!(!p.tabs_forbidden);
        assert_eq!(p.indent_size, 4);
    }

    #[test]
    fn test_load_profile_minimal() {
        let p = load_profile("minimal").unwrap();
        assert_eq!(p.keyword_casing, KeywordCasing::Preserve);
        assert!(!p.tabs_forbidden);
        assert_eq!(p.max_blank_lines, 2);
        assert!(!p.insert_first_comment);
    }

    #[test]
    fn test_unknown_profile() {
        assert!(matches!(
            load_profile("nonexistent"),
            Err(ConfigError::UnknownProfile(_))
        ));
    }

    #[test]
    fn test_keyword_casing_serde() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rexxlint.toml");
        fs::write(
            &config_path,
            "[formatting]\nname = \"custom\"\nline_length_soft = 80\nline_length_hard = 100\ntabs_forbidden = false\nkeyword_casing = \"lower\"\n",
        )
        .unwrap();
        let config = load_config(&config_path).unwrap();
        let fmt = config.formatting.unwrap();
        assert_eq!(fmt.keyword_casing, KeywordCasing::Lower);
    }

    #[test]
    fn test_files_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rexxlint.toml");
        fs::write(
            &config_path,
            "[files]\nexclude = [\"vendor/**\", \"generated/**\"]\n",
        )
        .unwrap();
        let config = load_config(&config_path).unwrap();
        let files = config.files.unwrap();
        assert_eq!(files.exclude, vec!["vendor/**", "generated/**"]);
    }
}
