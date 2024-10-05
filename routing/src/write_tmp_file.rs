use polars::error::PolarsError;
use polars::frame::DataFrame;
use polars::prelude::ParquetWriter;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

pub(crate) fn write_tmp_file(
    path: PathBuf,
    df: &mut DataFrame
) -> Result<(), PolarsError> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    
    let mut file = File::create(path)?;
    ParquetWriter::new(&mut file).finish(df)?;
    
    Ok(())
}