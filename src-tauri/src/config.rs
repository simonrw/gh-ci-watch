use std::path::Path;

use color_eyre::eyre::{self, Context};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub enable_sentry: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enable_sentry: true,
        }
    }
}

impl AppConfig {
    pub fn from_default_path() -> eyre::Result<Self> {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("gh-actions-monitor"))
            .ok_or(eyre::eyre!("no XDG config path defined"))?;
        let config_file = config_dir.join("config.toml");
        Self::from_path(config_file)
    }

    pub fn from_path(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("reading config file from path {}", path.display()))?;
        let config: Self = toml::from_str(&contents).wrap_err("parsing config file as toml")?;
        Ok(config)
    }
}
