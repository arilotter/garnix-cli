use thiserror::Error;

#[derive(Error, Debug)]
pub enum GarnixError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not in a git repository")]
    NotInGitRepo,

    #[error("No flake.nix found")]
    NoFlakeFound,

    #[error("Nix command failed: {0}")]
    NixCommand(String),

    #[error("Pattern matching error: {0}")]
    PatternMatch(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, GarnixError>;
