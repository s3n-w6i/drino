mod gtfs;

use std::{fmt, io};
use std::fmt::Display;
use std::path::PathBuf;
use polars::prelude::LazyFrame;
use crate::dataset::{Dataset, DatasetFormat};
use crate::preprocessing_steps::step1_fetch_data::FetchStepOutput;
use crate::preprocessing_steps::step2_import_data::gtfs::import_gtfs_data;
use crate::preprocessing_steps::step3_validate_data::rule_severity::Severity;
use crate::preprocessing_steps::step3_validate_data::rule_violations::RuleViolations;
use crate::preprocessing_steps::step3_validate_data::rules::Rule;

pub async fn import_data(
    prev_step_out: FetchStepOutput
) -> Result<ImportStepOutput, ImportError> {
    match prev_step_out.dataset.format {
        DatasetFormat::Gtfs => {
            let result = import_gtfs_data(prev_step_out).await?;
            Ok(result)
        }
        DatasetFormat::GtfsRt => {
            todo!("GTFS RT is not yet supported")
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ImportError {
    Zip(#[from] zip::result::ZipError),
    File(#[from] io::Error),
    Polars(#[from] polars::error::PolarsError),
    PathPersist(#[from] tempfile::PathPersistError),
    //RuleViolations(Vec<Box<dyn RuleViolations/*<dyn Rule<dyn Severity>, dyn Severity>*/>>)
}

impl Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            ImportError::Zip(err) => err,
            ImportError::File(err) => err,
            ImportError::Polars(err) => err,
            ImportError::PathPersist(err) => err,
            //ImportError::RuleViolations(violations) => violations
        };
        write!(f, "{}", err)
    }
}

pub struct ImportStepOutput {
    pub(crate) dataset: Dataset,
    pub(crate) extra: ImportStepExtra,
}

#[derive(Clone)]
pub enum ImportStepExtra {
    Gtfs {
        calendar: LazyFrame,
        stops: LazyFrame,
        trips: LazyFrame,
        stop_times: LazyFrame,
        temporary_files: Vec<PathBuf>
    }
}
