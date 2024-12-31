use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use polars::datatypes::AnyValue;
use polars::error::{PolarsError, PolarsResult};
use polars::frame::DataFrame;
use polars::io::SerWriter;
use polars::prelude::{CsvWriter, IpcWriter, LazyFrame, ParquetWriter};

pub fn count(frame: LazyFrame) -> PolarsResult<u32> {
        let count = frame.count()
            .collect()?;
        match count[0].get(0)? {
            AnyValue::UInt32(count) => Ok(count),
            _ => Err(PolarsError::ComputeError("Count was not u32".into()))
        }
}

pub enum FileType {
    CSV,
    IPC,
    PARQUET,
}

pub fn write_df_to_file(
    path: PathBuf,
    format: FileType,
    mut df: DataFrame
) -> Result<(), PolarsError> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;

    match format {
        FileType::CSV => {
            CsvWriter::new(&mut file).finish(&mut df)?;
            Ok(())
        },
        FileType::IPC => {
            IpcWriter::new(&mut file).finish(&mut df)?;
            Ok(())
        },
        FileType::PARQUET => {
            ParquetWriter::new(&mut file).finish(&mut df)?;
            Ok::<(), PolarsError>(())
        },
    }?;

    Ok(())
}