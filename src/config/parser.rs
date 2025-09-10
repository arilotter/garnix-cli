use crate::config::types::GarnixConfig;
use crate::error::Result;
use std::path::Path;

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<GarnixConfig> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(GarnixConfig::default());
    }

    let contents = std::fs::read_to_string(path)?;
    let config: GarnixConfig = serde_yaml::from_str(&contents)?;

    Ok(config)
}

pub fn load_config_from_current_dir() -> Result<GarnixConfig> {
    load_config("garnix.yaml")
}
