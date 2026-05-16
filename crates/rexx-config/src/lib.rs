use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormattingProfile {
    pub name: &'static str,
    pub line_length_soft: usize,
    pub line_length_hard: usize,
    pub uppercase_keywords: bool,
    pub tabs_forbidden: bool,
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("unknown formatting profile: {0}")]
    UnknownProfile(String),
}

pub const MAINFRAME_COMPATIBLE: FormattingProfile = FormattingProfile {
    name: "mainframe-compatible",
    line_length_soft: 72,
    line_length_hard: 80,
    uppercase_keywords: true,
    tabs_forbidden: true,
};

pub fn load_profile(name: &str) -> Result<FormattingProfile, ProfileError> {
    match name {
        "mainframe-compatible" => Ok(MAINFRAME_COMPATIBLE),
        _ => Err(ProfileError::UnknownProfile(name.to_string())),
    }
}

pub fn default_profile() -> FormattingProfile {
    MAINFRAME_COMPATIBLE
}

#[cfg(test)]
mod tests {
    use super::{MAINFRAME_COMPATIBLE, default_profile, load_profile};

    #[test]
    fn loads_mainframe_profile() {
        let p = load_profile("mainframe-compatible").expect("profile exists");
        assert_eq!(p, MAINFRAME_COMPATIBLE);
    }

    #[test]
    fn default_is_mainframe() {
        assert_eq!(default_profile(), MAINFRAME_COMPATIBLE);
    }
}
