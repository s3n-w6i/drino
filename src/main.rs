mod gtfs_file;
mod config;
mod dataset;
mod preprocessing_steps;
mod routing;

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use tempfile::TempPath;
use crate::config::Config;
use crate::preprocessing_steps::step1_fetch_data::{fetch_dataset, FetchError};
use crate::preprocessing_steps::step2_import_data::{import_data, ImportError, ImportStepExtra};
use crate::preprocessing_steps::step3_validate_data::{validate_data, ValidateError, ValidateStepOutput};
use crate::preprocessing_steps::step4_merge_data::{merge, MergeError};
use crate::preprocessing_steps::step5_simplify::{simplify, SimplifyError};
use crate::routing::algorithm::RoutingAlgorithm;
use crate::routing::stp::ScalableTransferPatternsAlgorithm;

type ALGORITHM = ScalableTransferPatternsAlgorithm;

#[tokio::main]
async fn main() -> Result<(), DrinoError> {
    env_logger::init();
    dbg!(std::env::temp_dir());
    let start_time = SystemTime::now();

    let config_file = File::open("config.yaml").expect("config.yaml was not provided.");
    let config: Config = serde_yaml::from_reader(config_file).expect("Could not read config.yaml");

    let result: Result<(), DrinoError> = match config {
        Config::Version1 { datasets, .. } => {
            let mut files_to_clean_up: Vec<PathBuf> = vec![];

            // TODO: Process all datasets
            let datasets = datasets.into_iter().take(1);

            let results = futures::stream::iter(datasets)
                .then(|dataset| async move {
                    let fetch_out = fetch_dataset(dataset).await?;
                    let import_out = import_data(fetch_out).await?;
                    let validated = validate_data(import_out).await?;
                    Ok::<ValidateStepOutput, DrinoError>(validated)
                })
                .inspect_err(|err| {
                    eprintln!("{}", err);
                })
                .collect::<Vec<Result<ValidateStepOutput, DrinoError>>>()
                .await
                .into_iter()
                .collect::<Result<Vec<ValidateStepOutput>, DrinoError>>()?;

            results.iter().for_each(|result| {
                match &result.extra { ImportStepExtra::Gtfs { temporary_files, .. } => {
                    temporary_files.iter().for_each(|f| files_to_clean_up.push(f.clone()))
                } }
            });

            let merged = merge(results).await?;
            let simplified = simplify(merged).await?;
            // TODO: Merge datasets (with deduplication) and frequency reduce calender times
            let preprocessing_result = ALGORITHM::preprocess(simplified).await;

            println!("Preprocessing took {:?}", start_time.elapsed().unwrap());
            files_to_clean_up.into_iter()
                .for_each(|file| {
                    TempPath::from_path(file).close()
                        .expect("Unable to clean up temp files. Pleas clean up manually.");
                });
            Ok(())
        }
    };

    result
}

#[derive(thiserror::Error, Debug)]
pub enum DrinoError {
    Fetch(#[from] FetchError),
    Import(#[from] ImportError),
    Validate(#[from] ValidateError),
    Merge(#[from] MergeError),
    Simplify(#[from] SimplifyError),
}

impl Display for DrinoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err: &dyn Display = match self {
            DrinoError::Fetch(err) => err,
            DrinoError::Import(err) => err,
            DrinoError::Validate(err) => err,
            DrinoError::Merge(err) => err,
            DrinoError::Simplify(err) => err,
        };
        write!(f, "{}", err)
    }
}