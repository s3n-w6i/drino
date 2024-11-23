use polars::error::PolarsError;
use std::fmt::{Display, Formatter};

#[derive(thiserror::Error, Debug)]
pub(super) enum TransferPatternConstructionError {
    Polars(#[from] PolarsError)
}

impl Display for TransferPatternConstructionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { 
            TransferPatternConstructionError::Polars(err) => {
                write!(f, "{}", err)
            }
        }
    }
}