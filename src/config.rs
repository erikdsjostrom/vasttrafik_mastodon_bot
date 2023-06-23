use anyhow::Result;
use elefren::data::Data;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.toml";

/// Mastodon config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mastodon: Data,
    last_status_id: String,
}

impl Config {
    /// Create a new config
    pub fn new() -> Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(CONFIG_FILE)?)?)
    }

    /// Get the last status id
    pub fn get_last_status_id(&self) -> String {
        self.last_status_id.clone()
    }

    /// Set the last status id
    pub fn set_last_status_id(&mut self, id: String) {
        self.last_status_id = id;
    }

    /// Save the config
    /// Useful for updating the last status id
    pub fn save(&self) -> Result<()> {
        std::fs::write(CONFIG_FILE, toml::to_string(self)?)?;
        Ok(())
    }
}
