use crate::transfers::TransferError;
use serde::{Serialize, Serializer};
use std::fmt;
use std::fmt::Display;

pub type QueryResult<O> = Result<O, QueryError>;
pub type MultiQueryResult<O> = Result<Vec<O>, QueryError>;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    Polars(#[from] polars::error::PolarsError),
    NoRouteFound,
    TransferError(#[from] TransferError),
}

impl Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            QueryError::Polars(err) => err,
            QueryError::NoRouteFound => &"No route found",
            QueryError::TransferError(err) => err,
        };
        write!(f, "{}", err)
    }
}

impl Serialize for QueryError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
