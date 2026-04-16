use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtupaConfig {
    pub rpc_url: String,
    pub etherscan_key: Option<String>,
    pub output_dir: String,
    pub studio_dir: Option<PathBuf>,
}

impl Default for AtupaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8547".to_string(),
            etherscan_key: None,
            output_dir: ".".to_string(),
            studio_dir: None,
        }
    }
}

impl AtupaConfig {
    /// Load configuration by merging multiple sources.
    /// Priority: CLI Flags (applied later) > Env Vars > atupa.toml > ~/.atupa/config.toml > Defaults
    pub fn load() -> Self {
        let mut figment = Figment::from(Serialized::defaults(Self::default()));

        // Global config
        if let Some(mut home) = dirs::home_dir() {
            home.push(".atupa");
            home.push("config.toml");
            figment = figment.merge(Toml::file(home));
        }

        // Local config
        figment = figment.merge(Toml::file("atupa.toml"));

        // Environment variables
        figment = figment.merge(Env::prefixed("ATUPA_"));

        figment.extract().unwrap_or_else(|_| Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = AtupaConfig::default();
        assert_eq!(config.rpc_url, "http://localhost:8547");
        assert!(config.etherscan_key.is_none());
    }

    #[test]
    fn test_env_override() {
        unsafe {
            env::set_var("ATUPA_RPC_URL", "http://test-rpc.local");
            env::set_var("ATUPA_ETHERSCAN_KEY", "test-key-123");
        }
        
        // Reloading should pick up env vars due to Env::prefixed("ATUPA_")
        let config = AtupaConfig::load();
        
        assert_eq!(config.rpc_url, "http://test-rpc.local");
        assert_eq!(config.etherscan_key, Some("test-key-123".to_string()));
        
        unsafe {
            env::remove_var("ATUPA_RPC_URL");
            env::remove_var("ATUPA_ETHERSCAN_KEY");
        }
    }
}
