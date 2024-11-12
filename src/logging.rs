use log::LevelFilter;

pub(super) fn initialize_logging() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .parse_default_env() // Allow overriding log level through RUST_LOG env var
        .init();
}