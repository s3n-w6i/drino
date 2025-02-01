pub mod bootstrap_config;
mod config;
mod preprocessing;

use crate::config::load_config;
use bootstrap_config::BootstrapConfig;
use common::types::config::Config;
use common::util::logging;
use common::util::speed::Speed;
use data_harvester::step1_fetch_data::FetchError;
use data_harvester::step2_import_data::ImportError;
use data_harvester::step3_validate_data::ValidateError;
use data_harvester::step4_merge_data::MergeError;
use data_harvester::step5_simplify::SimplifyError;
use log::{debug, error, info};
use polars::error::PolarsError;
use routing::algorithm::PreprocessingError;
use routing::stp::ScalableTransferPatternsAlgorithm;
use std::fmt::{Display, Formatter};
use std::thread;
use preprocessing::preprocess;

type ALGORITHM = ScalableTransferPatternsAlgorithm;

// The maximum speed in km/h that any vehicle can travel
// This must be high enough, otherwise wrong routes might be calculated
pub const MAX_SPEED: Speed = Speed(500.0);

fn main() {
    let _ = run().inspect_err(|err| error!("{}", err));
}

fn run() -> Result<(), DrinoError> {
    let bootstrap_config = BootstrapConfig::read();

    logging::init(bootstrap_config.clone().log_level.into());
    print_startup_message();

    debug!(target: "main", "Using temporary folder at {}", std::env::temp_dir().to_str().unwrap());

    let config = load_config(bootstrap_config)?;
    
    info!(target: "visualization", "Launching visualization server");
    let vis_server_config = config.clone();
    let vis_server_thread = thread::spawn(move || {
        let vis_server_rt = actix_web::rt::Runtime::new()
            .expect("Could not create actix runtime");
        let vis_server_handle = vis_server_rt.spawn(async move {
            let vis_server = visualization::build_server(
                vis_server_config, "./data".into(), true
            ).await.expect("Error building visualization server");
            
            vis_server.await.expect("Error running visualization server");
        });
        
        vis_server_rt.block_on(vis_server_handle).unwrap();
        
        info!(target: "visualization", "Visualization server shut down");
    });

    match config {
        Config::Version1 { datasets, .. } => {
            let algorithm = preprocess(datasets)?;

            serve(algorithm)?;
        }
    };
    
    vis_server_thread.join().expect("Visualization server thread join error");

    Ok(())
}

fn print_startup_message() {
    info!("\n      _      _             \n   __| |_ __(_)_ __   ___  \n  / _` | '__| | '_ \\ / _ \\ \n | (_| | |  | | | | | (_) |\n  \\__,_|_|  |_|_| |_|\\___/ \n                           \n R O U T I N G   E N G I N E\n");
}

fn serve(algorithm: ALGORITHM) -> Result<(), DrinoError> {
    let rt = actix_web::rt::Runtime::new()
        .expect("Unable to create server runtime");

    let server_handle = rt.spawn(async move {
        /*HttpServer::new(move || {})
            .bind("127.0.0.1:8080");*/
    });

    rt.block_on(server_handle).unwrap();


    todo!()
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
        };
        let prefix = match self {
            DrinoError::Config(_) => "Error while reading config file",
            DrinoError::Fetch(_) => "Error while fetching a dataset",
            DrinoError::Import(_) => "Error while fetching a dataset",
            DrinoError::Validate(_) => "Error while validating a dataset",
            DrinoError::Merge(_) => "Error while merging datasets",
            DrinoError::Simplify(_) => "Error while simplifying a dataset",
            DrinoError::Polars(_) => "Error while processing dataset data",
            DrinoError::Preprocessing(_) => "Error while preprocessing data",
            DrinoError::IO(_) => "Error during IO",
        };
        write!(f, "{}: {}", prefix, err)
    }
}
