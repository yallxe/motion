use std::path::Path;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub downstreams: Vec<DownstreamConfig>,
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownstreamConfig {
    pub address: String,
    pub name: String,
    pub default: bool
}

impl Configuration {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let config: Configuration = serde_yaml::from_reader(
            std::fs::File::open(path)?
        )?;

        Ok(config)
    }
}