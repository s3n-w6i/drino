use log::LevelFilter;
use clap::Parser;

#[derive(Parser, Clone)]
#[command(version, about)]
pub struct BootstrapConfig {
    #[clap(short('c'), long("config"), env("DRINO_CONFIG"), default_value_os = "config.json")]
    pub config_file: String,
    #[clap(short('l'), long("log-level"), env("DRINO_LOG_LEVEL"), default_value_t, value_enum)]
    pub log_level: LogLevel,
}

impl BootstrapConfig {
    pub fn read() -> Self {
        BootstrapConfig::parse()
    }
}


#[derive(clap::ValueEnum, Clone, Default)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => Self::Off,
            LogLevel::Error => Self::Error,
            LogLevel::Warn => Self::Warn,
            LogLevel::Info => Self::Info,
            LogLevel::Debug => Self::Debug,
            LogLevel::Trace => Self::Trace,
        }
    }
}