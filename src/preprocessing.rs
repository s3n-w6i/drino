use std::path::PathBuf;
use std::time::SystemTime;
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use polars::prelude::IntoLazy;
use tempfile::TempPath;
use tokio::runtime::Runtime;
use common::types::dataset::Dataset;
use common::util::df::{write_geoarrow_to_file, FileType};
use common::util::logging;
use data_harvester::step1_fetch_data::fetch_dataset;
use data_harvester::step2_import_data::{import_data, ImportStepExtra};
use data_harvester::step3_validate_data::{validate_data, ValidateStepOutput};
use data_harvester::step4_merge_data::merge;
use data_harvester::step5_simplify::simplify;
use routing::algorithm::{PreprocessInit, PreprocessingError, PreprocessingInput};
use routing::direct_connections::DirectConnections;
use crate::{DrinoError, ALGORITHM};
use crate::config::ConfigError;

/// Wrapper for `preprocess_inner` that handles cleaning up temporary files, even if error was
/// thrown.
pub fn preprocess(datasets: Vec<Dataset>) -> Result<ALGORITHM, DrinoError> {
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
                match datasets.len() {
                    0 => {
                        Err(DrinoError::Config(ConfigError::NoDatasets()))
                    }
                    2.. => {
                        todo!("Using multiple datasets is not yet supported")
                    },
                    1 => {
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
                    }
                }
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

/// Cleans up files that were created during preprocessing
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
