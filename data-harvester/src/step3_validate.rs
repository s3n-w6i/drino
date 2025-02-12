use std::fmt::Display;
use std::io::ErrorKind;
use std::process::{Command, Stdio};
use common::types::dataset::{DataSource, Dataset, DatasetFormat};
use log::{error, warn};
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
            return Err(GtfsTidy("GtfsTidy command was not found. Please install it or disable feed validation."));            
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
        .arg(filepath)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    let status = gtfstidy_out.status.code()
        .ok_or(GtfsTidy("Failed to receive status code from GtfsTidy"));

    match status {
        Ok(0) => Ok(()),
        Err(e) => Err(e),
        _ => Err(GtfsTidy("GtfsTidy failed"))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ValidateError {
    UnknownFormat,
    UnsupportedDatasetSource,
    IO(#[from] std::io::Error),
    GtfsTidy(&'static str),
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