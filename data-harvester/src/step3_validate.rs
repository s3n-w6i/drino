use std::fmt::Display;
use std::io::{BufRead, ErrorKind};
use std::process::Command;
use common::types::dataset::{DataSource, Dataset, DatasetFormat};
use log::{debug, error, warn};
use crate::step2_import::{ImportStepExtra, ImportStepOutput};
use crate::step3_validate::ValidateError::{GtfsTidy, UnknownFormat, UnsupportedDatasetSource};

pub async fn validate_data(
    ImportStepOutput { dataset, extra }: ImportStepOutput
) -> Result<ValidateStepOutput, ValidateError> {
    check_existence_of_tools()?;

    let result = match dataset.format {
        DatasetFormat::Gtfs => {
            validate_gtfs(&dataset).await
        },
        _ => {
            return Err(UnknownFormat)
        }
    };

    let skip = match result {
        Ok(_) => { false }
        Err(e) => {
            error!(target: "validation", "Error in dataset '{}': {}", dataset.id, e);
            warn!(target: "validation", "Skipping dataset '{}' because of validation errors", dataset.id);
            true
        }
    };

    Ok(ValidateStepOutput {
        dataset,
        extra,
        skip
    })
}

fn check_existence_of_tools() -> Result<(), ValidateError> {
    let gtfstidy_dummy_call = Command::new("gtfstidy").output();

    if let Err(e) = gtfstidy_dummy_call {
        if let ErrorKind::NotFound = e.kind() {
            return Err(GtfsTidy("GtfsTidy command was not found. Please install it or disable feed validation.".into()));
        }
    }

    Ok(())
}

async fn validate_gtfs(dataset: &Dataset) -> Result<(), ValidateError> {
    let filepath = match &dataset.src {
        DataSource::URL { .. } => {
            return Err(UnsupportedDatasetSource)
        }
        DataSource::File { path } => path
    };

    let gtfstidy_out = Command::new("gtfstidy")
        .arg("-v")
        .arg(filepath)
        .output()?;
    
    let status = gtfstidy_out.status.code();

    match status {
        None => { return Err(GtfsTidy("Unable to get GtfsTidy's status code".into()))}
        Some(0) => {}, // All good, continue printing normal output
        Some(_) => {
            // Collect gtfstidy's error output and put it into the error message
            let error_message = gtfstidy_out.stderr.lines()
                .map(|line| {
                    line.unwrap_or_else(|e| format!("Unable to error output line: {e}"))
                })
                .collect::<Vec<String>>()
                .join("\n");
            return Err(GtfsTidy(error_message));
        }
    }

    // Forward standard output of gtfstidy
    gtfstidy_out.stdout.lines().for_each(|line| {
        match line {
            Ok(line) => { debug!(target: "gtfstidy", "{}", line); }
            Err(e) => { warn!(target: "gtfstidy", "{}", e); }
        }
    });

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ValidateError {
    UnknownFormat,
    UnsupportedDatasetSource,
    IO(#[from] std::io::Error),
    GtfsTidy(String),
}

impl Display for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownFormat => write!(f, "Unknown format"),
            UnsupportedDatasetSource => write!(f, "Unsupported DatasetSource"),
            ValidateError::IO(e) => write!(f, "IO error: {e}"),
            GtfsTidy(e) => write!(f, "GtfsTidy failed: {e}"),
        }
    }
}

pub struct ValidateStepOutput {
    pub(crate) dataset: Dataset,
    pub extra: ImportStepExtra,
    pub(crate) skip: bool
}