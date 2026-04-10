use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtupaConfig {
    pub rpc_url: String,
    pub etherscan_key: Option<String>,
    pub output_dir: String,
}

impl Default for AtupaConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            etherscan_key: None,
            output_dir: ".".to_string(),
        }
    }
}

impl AtupaConfig {
    /// Load configuration by merging multiple sources.
    /// Priority: CLI Flags > Env Vars > atupa.toml > Defaults
    pub fn load() -> Self {
        Figment::from(Serialized::defaults(Self::default()))
            .merge(Toml::file("atupa.toml"))
            .merge(Env::prefixed("ATUPA_"))
            .extract()
            .unwrap_or_else(|_| Self::default())
    }
}
