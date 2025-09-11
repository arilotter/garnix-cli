use crate::config::types::GarnixConfig;
use crate::error::Result;
use std::path::Path;

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Option<GarnixConfig>> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)?;
    let config: GarnixConfig = serde_yaml::from_str(&contents)?;

    Ok(Some(config))
}

pub fn load_config_from_git_root<P: AsRef<Path>>(git_root: P) -> Result<Option<GarnixConfig>> {
    let config_path = git_root.as_ref().join("garnix.yaml");
    load_config(config_path)
}
