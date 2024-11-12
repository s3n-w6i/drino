use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub(super) struct BootstrapConfig {
    #[clap(short('c'), long("config"), env("DRINO_CONFIG"), default_value_os = "config.yaml")]
    pub(super) config_file: String
}

impl BootstrapConfig {
    pub(super) fn read() -> Self {
        BootstrapConfig::parse()
    }
}