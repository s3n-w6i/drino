use common::types::config::Config;
use log::info;
use std::fs::File;
use std::path::Path;
use crate::bootstrap_config::BootstrapConfig;
use crate::DrinoError;

pub(super) fn load_config(bootstrap_config: BootstrapConfig) -> Result<Config, DrinoError> {
    let path: &Path = &Path::new(&bootstrap_config.config_file);
    
    let config_file = File::open(path)?;
    let config: Config = serde_yaml::from_reader(config_file)
        .expect(&format!("Could not read Config file '{path:?}'"));

    info!(target: "main", "Config read successfully from '{path:?}'");

    Ok(config)
}