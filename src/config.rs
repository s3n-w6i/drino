use std::fmt::Display;
use common::types::config::Config;
use log::{debug, info};
use std::fs::File;
use std::io;
use std::path::Path;
use crate::bootstrap_config::BootstrapConfig;

pub(super) fn load_config(bootstrap_config: BootstrapConfig) -> Result<Config, ConfigError> {
    let path: &Path = &Path::new(&bootstrap_config.config_file);
    
    let config_file = File::open(path)?;
    let config_extension = path.extension();
    
    if let Some(extension) = config_extension {
        let config: Config = match extension.to_str() { 
            Some("yml") | Some("yaml") => {
                serde_yml::from_reader(config_file)?
            },
            Some("json") => {
                serde_json::from_reader(config_file)?
            },
            _ => {
                return Err(ConfigError::UnknownFileExtension())
            }
        };

        info!(target: "main", "Config read successfully from {path:?}");
        debug!(target: "main", "Using config: {:?}", config);

        Ok(config)
    } else {
        Err(ConfigError::MissingFileExtension())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    IO(#[from] io::Error),
    DeserializationYaml(#[from] serde_yml::Error),
    DeserializationJson(#[from] serde_json::Error),
    MissingFileExtension(),
    UnknownFileExtension(),
    NoDatasets()
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::IO(err) => write!(f, "{}", err),
            ConfigError::DeserializationYaml(err) => write!(f, "{}", err),
            ConfigError::DeserializationJson(err) => write!(f, "{}", err),
            ConfigError::MissingFileExtension() => write!(f, "File extension not provided. Please provide .yml, .yaml or .json in the file path."),
            ConfigError::UnknownFileExtension() => write!(f, "File extension not recognized. Please provide .yml, .yaml or .json in the file path."),
            ConfigError::NoDatasets() => write!(f, "No datasets provided."),
        }?;
        
        Ok(())
    }
}