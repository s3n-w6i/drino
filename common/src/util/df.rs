use geoarrow::error::GeoArrowError;
use geoarrow::io::ipc::write_ipc;
use geoarrow::table::Table;
use itertools::Itertools;
use polars::datatypes::AnyValue;
use polars::error::{PolarsError, PolarsResult};
use polars::frame::DataFrame;
use polars::io::SerWriter;
use polars::prelude::{col, CsvWriter, IntoLazy, IpcWriter, LazyFrame, ParquetWriter, SortMultipleOptions};
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

pub fn count(frame: LazyFrame) -> PolarsResult<u32> {
        let count = frame.count()
            .collect()?;
        match count[0].get(0)? {
            AnyValue::UInt32(count) => Ok(count),
            _ => Err(PolarsError::ComputeError("Count was not u32".into()))
        }
}

/// Compare two dataframes without regard to the ordering of columns and/or rows
pub fn equivalent(lhs: &DataFrame, rhs: &DataFrame, ignore_col_order: bool, ignore_row_order: bool) -> PolarsResult<bool> {
    fn normalize_col_order(frame: &DataFrame) -> PolarsResult<DataFrame> {
        frame.clone().lazy()
            .select( // Select all columns, but with a specific order
                frame.get_column_names().into_iter()
                    .sorted() // This sorting step ensures the same ordering
                    .map(|n| col(n.clone()))
                    .collect_vec(),
            )
            .collect()
    }
    
    fn normalize_row_order(frame: &DataFrame) -> PolarsResult<DataFrame> {
        frame.clone().lazy()
            // Sort by all columns
            .sort(
                frame.get_columns().into_iter()
                    // Lists are not sortable. This is a hack, since order of lists might be important.
                    .filter(|col| !col.dtype().is_list())
                    .map(|col| col.name()).cloned()
                    .collect_vec(),
                SortMultipleOptions::default()
            )
            .collect()
    }

    let (lhs, rhs) = if ignore_col_order {
        (&normalize_col_order(lhs)?, &normalize_col_order(rhs)?)
    } else { (lhs, rhs) };

    let (lhs, rhs) = if ignore_row_order {
        (&normalize_row_order(lhs)?, &normalize_row_order(rhs)?)
    } else { (lhs, rhs) };
    
    let equivalent = lhs.eq(rhs);

    Ok(equivalent)
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
    let mut file = prepare_file(path)?;

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

pub fn write_geoarrow_to_file(
    path: PathBuf,
    format: FileType,
    table: Table
) -> Result<(), GeoArrowError> {
    let file = prepare_file(path)?;
    
    match format {
        FileType::IPC => {
            write_ipc(table.into_record_batch_reader(), file)?;
        }
        _ => {
            panic!("Unsupported file type");
        }
    }
    
    Ok(())
}


fn prepare_file(
    path: PathBuf,
) -> Result<File, std::io::Error> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    
    Ok(file)
}