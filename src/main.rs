pub mod bootstrap_config;
mod config;

use crate::config::load_config;
use bootstrap_config::BootstrapConfig;
use common::types::config::Config;
use common::types::dataset::Dataset;
use common::util::df::{write_geoarrow_to_file, FileType};
use common::util::logging;
use common::util::speed::Speed;
use data_harvester::step1_fetch_data::{fetch_dataset, FetchError};
use data_harvester::step2_import_data::{import_data, ImportError, ImportStepExtra};
use data_harvester::step3_validate_data::{validate_data, ValidateError, ValidateStepOutput};
use data_harvester::step4_merge_data::{merge, MergeError};
use data_harvester::step5_simplify::{simplify, SimplifyError};
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use polars::error::PolarsError;
use polars::prelude::IntoLazy;
use routing::algorithm::{PreprocessInit, PreprocessingError, PreprocessingInput};
use routing::direct_connections::DirectConnections;
use routing::stp::ScalableTransferPatternsAlgorithm;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::thread;
use std::time::SystemTime;
use tempfile::TempPath;
use tokio::runtime::Runtime;

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
                vis_server_config, "./data".into()
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

/// Wrapper for `preprocess_inner` that handles cleaning up temporary files, no matter if error is thrown or not.
fn preprocess(datasets: Vec<Dataset>) -> Result<ALGORITHM, DrinoError> {
    let mut files_to_clean_up: Vec<PathBuf> = vec![];

    let result = preprocess_inner(datasets, &mut files_to_clean_up);

    clean_up(files_to_clean_up);

    result
}

fn preprocess_inner(
    datasets: Vec<Dataset>,
    files_to_clean_up: &mut Vec<PathBuf>,
) -> Result<ALGORITHM, DrinoError> {
    info!(target: "preprocessing", "Starting preprocessing");
    let preprocessing_start_time = SystemTime::now();

    let preprocessing_input =
        logging::run_with_spinner("preprocessing", "Fetching and importing datasets", || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                if datasets.len() > 1 {
                    todo!("Using multiple datasets is not yet supported")
                }
                let datasets = datasets.into_iter().take(1);

                let results = futures::stream::iter(datasets)
                    .then(|dataset| async move {
                        let fetch_out = fetch_dataset(dataset).await?;
                        let import_out = import_data(fetch_out).await?;
                        let validated = validate_data(import_out).await?;
                        Ok::<ValidateStepOutput, DrinoError>(validated)
                    })
                    .inspect_err(|err| {
                        error!("{}", err);
                    })
                    .collect::<Vec<Result<ValidateStepOutput, DrinoError>>>()
                    .await
                    .into_iter()
                    .collect::<Result<Vec<ValidateStepOutput>, DrinoError>>()?;

                results.iter().for_each(|result| match &result.extra {
                    ImportStepExtra::Gtfs {
                        temporary_files, ..
                    } => temporary_files
                        .iter()
                        .for_each(|f| files_to_clean_up.push(f.clone())),
                });

                let merged = merge(results).await?;
                let simplified = simplify(merged).await?;

                Ok::<PreprocessingInput, DrinoError>(simplified)
            })
        })?;

    // TODO: Merge datasets (with deduplication) and frequency reduce calender times

    // Cache important (and small) tables like stops to speed up computation
    let cached_input = logging::run_with_spinner(
        "preprocessing",
        "Reading and caching timetable data",
        move || {
            Ok::<PreprocessingInput, DrinoError>(PreprocessingInput {
                stops: preprocessing_input.stops.collect()?.lazy(),
                stop_times: preprocessing_input.stop_times.collect()?.lazy(),
                ..preprocessing_input
            })
        },
    )?;

    // Build visualization of lines
    logging::run_with_spinner("visualization", "Building visualization for lines", || {
        let direct_connections = DirectConnections::try_from(cached_input.clone())?;
        let table = direct_connections
            .to_geoarrow_lines(cached_input.stops.clone())
            .map_err(|e| PreprocessingError::BuildLines(e))?;

        write_geoarrow_to_file("./data/tmp/global/lines.arrow".into(), FileType::IPC, table)
            .map_err(|e| PreprocessingError::GeoArrow(e))?;

        Ok::<(), DrinoError>(())
    })?;

    let preprocessing_result = ALGORITHM::preprocess(cached_input, true)?;

    let elapsed = indicatif::HumanDuration(preprocessing_start_time.elapsed().unwrap());
    info!(target: "preprocessing", "Preprocessing finished in {}", elapsed);

    Ok(preprocessing_result)
}

fn clean_up(files: Vec<PathBuf>) {
    if !files.is_empty() {
        files.into_iter().for_each(|file| {
            TempPath::from_path(file.clone()).close().expect(
                format!("Unable to clean up temp file at {file:?}. Please clean up manually.")
                    .as_str(),
            );
        });

        debug!("Temporary files cleaned up");
    }
}

fn serve(algorithm: ALGORITHM) -> Result<(), DrinoError> {
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
