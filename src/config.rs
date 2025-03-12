use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub github_token: Option<String>,
    pub github_username: Option<String>,
    pub openai_api_key: Option<String>,
    pub default_list: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github_token: None,
            github_username: None,
            openai_api_key: None,
            default_list: None,
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config: Config = confy::load("gh-stars", None)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    confy::store("gh-stars", None, config)?;
    Ok(())
}

pub fn get_config_path() -> Result<PathBuf> {
    let config_path = confy::get_configuration_file_path("gh-stars", None)?;
    Ok(config_path)
}