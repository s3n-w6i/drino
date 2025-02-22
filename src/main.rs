pub mod bootstrap_config;
mod config;
mod preprocessing;

use crate::config::load_config;
use bootstrap_config::BootstrapConfig;
use common::types::config::Config;
use common::util::logging;
use common::util::speed::Speed;
use data_harvester::step1_fetch::FetchError;
use data_harvester::step2_import::ImportError;
use data_harvester::step3_validate::ValidateError;
use data_harvester::step4_merge::MergeError;
use data_harvester::step5_simplify::SimplifyError;
use log::{debug, error, info};
use polars::error::PolarsError;
use preprocessing::preprocess;
use routing::algorithm::PreprocessingError;
use routing::raptor::RaptorAlgorithm;
use routing::stp::ScalableTransferPatternsAlgorithm;
use std::fmt::{Display, Formatter};
use std::thread;
use tokio::signal;

type ALGORITHM = ScalableTransferPatternsAlgorithm;

// The maximum speed in km/h that any vehicle can travel
// This must be high enough, otherwise wrong routes might be calculated
pub const MAX_SPEED: Speed = Speed(500.0);

#[tokio::main]
async fn main() {
    let _ = run()
        .await
        .inspect_err(|err| error!(target: "main", "{}", err));
}

async fn run() -> Result<(), DrinoError> {
    let bootstrap_config = BootstrapConfig::read();

    logging::init(bootstrap_config.clone().log_level.into());
    print_startup_message();

    debug!(target: "main", "Using temporary folder at {}", std::env::temp_dir().to_str().unwrap());

    let config = load_config(bootstrap_config)?;

    info!(target: "visualization", "Launching visualization server");
    let vis_server = visualization::build_server(config.clone(), "./data".into(), true).await?;
    let vis_server_handle = vis_server.handle();
    tokio::spawn(vis_server);

    let api_server = match config {
        Config::Version1 { datasets, .. } => {
            let algorithm = preprocess(datasets).await?;

            server::build(algorithm).await?
        }
    };
    let api_server_handle = api_server.handle();
    tokio::spawn(api_server);

    signal::ctrl_c().await?;
    info!(target: "main", "Received shutdown signal");

    logging::run_with_spinner_async("main", "Shutting down servers", async || {
        vis_server_handle.stop(true).await;
        debug!(target: "main", "Visualization server stopped");
        api_server_handle.stop(true).await;
        debug!(target: "main", "API server stopped");
    })
    .await;

    Ok(())
}

fn print_startup_message() {
    info!("\n      _      _             \n   __| |_ __(_)_ __   ___  \n  / _` | '__| | '_ \\ / _ \\ \n | (_| | |  | | | | | (_) |\n  \\__,_|_|  |_|_| |_|\\___/ \n                           \n R O U T I N G   E N G I N E\n");
}

#[derive(thiserror::Error, Debug)]
pub enum DrinoError {
    Config(#[from] config::ConfigError),
    Fetch(#[from] FetchError),
    Import(#[from] ImportError),
    Validate(#[from] ValidateError),
    Merge(#[from] MergeError),
    Simplify(#[from] SimplifyError),
    Polars(#[from] PolarsError),
    Preprocessing(#[from] PreprocessingError),
    IO(#[from] std::io::Error),
    Server(#[from] server::ServerError),
}

impl Display for DrinoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err: &dyn Display = match self {
            DrinoError::Config(err) => err,
            DrinoError::Fetch(err) => err,
            DrinoError::Import(err) => err,
            DrinoError::Validate(err) => err,
            DrinoError::Merge(err) => err,
            DrinoError::Simplify(err) => err,
            DrinoError::Polars(err) => err,
            DrinoError::Preprocessing(err) => err,
            DrinoError::IO(err) => err,
            DrinoError::Server(err) => err,
        };
        let prefix = match self {
            DrinoError::Config(_) => "Reading config file",
            DrinoError::Fetch(_) => "Fetching datasets",
            DrinoError::Import(_) => "Importing datasets",
            DrinoError::Validate(_) => "Validating datasets",
            DrinoError::Merge(_) => "Merging datasets",
            DrinoError::Simplify(_) => "Simplifying datasets",
            DrinoError::Polars(_) => "Processing dataset data",
            DrinoError::Preprocessing(_) => "Preprocessing data",
            DrinoError::IO(_) => "Error during IO",
            DrinoError::Server(_) => "Error in server",
        };
        write!(f, "{}: {}", prefix, err)
    }
}
