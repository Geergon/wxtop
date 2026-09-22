use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{self, OptionExt};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub current_city: String,
}

pub fn init_config() -> color_eyre::Result<()> {
    let config_path = dirs::config_dir().ok_or_eyre("failed to get config dir path")?;
    let path = Path::new(&config_path).join("wxtop");
    if !path.exists() {
        fs::create_dir(&path)?;
    }

    let config_path = path.join("config.toml");
    if !config_path.exists() {
        let default_config = toml::to_string_pretty(&Config::default())?;
        fs::write(&config_path, default_config)?;
    }

    Ok(())
}

pub fn read_config() -> color_eyre::Result<()> {
    let config_path = dirs::config_dir()
        .ok_or_eyre("failed to get config dir path")?
        .join("wxtop/config.toml");

    if !config_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(())
}

pub fn write_config(config: &Config) -> color_eyre::Result<()> {
    let config_path = dirs::config_dir()
        .ok_or_eyre("failed to get config dir path")?
        .join("wxtop/config.toml");

    if !config_path.exists() {
        eyre::bail!("failed to find config file at {config_path:?}")
    }

    let toml_string = toml::to_string_pretty(config).expect("failed to serialize config");
    fs::write(config_path, toml_string)?;

    Ok(())
}
