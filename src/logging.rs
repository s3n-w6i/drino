use log::LevelFilter;
use crate::bootstrap_config::BootstrapConfig;

pub(super) fn initialize_logging(config: BootstrapConfig) {
    env_logger::builder()
        .filter_level(config.log_level.into())
        .parse_default_env() // Allow overriding log level through RUST_LOG env var
        .init();
}