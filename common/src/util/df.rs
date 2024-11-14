use polars::datatypes::AnyValue;
use polars::error::{PolarsError, PolarsResult};
use polars::prelude::LazyFrame;

pub fn count(frame: LazyFrame) -> PolarsResult<u32> {
        let count = frame.count()
            .collect()?;
        match count[0].get(0)? {
            AnyValue::UInt32(count) => Ok(count),
            _ => Err(PolarsError::ComputeError("Count was not u32".into()))
        }
}