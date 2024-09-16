use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use polars::datatypes::DataType;
use polars::prelude::{col, GetOutput, LazyCsvReader, LazyFileListReader, TimeUnit};
use tempfile::NamedTempFile;
use zip::ZipArchive;

use crate::step1_fetch_data::FetchStepOutput;
use crate::step2_import_data::{ImportError, ImportStepExtra, ImportStepOutput};
use crate::gtfs_file::{gtfs_date_format, GTFS_FILES_TO_IMPORT, GTFS_OTHER_FILES, GTFS_REQUIRED_FILES, gtfs_time_to_ms};

pub(crate) async fn import_gtfs_data(
    FetchStepOutput {
        path,
        dataset
    }: FetchStepOutput
) -> Result<ImportStepOutput, ImportError> {
    let mut zip_archive_file = File::open(path)?;
    let mut zip_archive = ZipArchive::new(&mut zip_archive_file)?;

    check_files_in_archive(&zip_archive)?;
    let extra = import_gtfs_files(&mut zip_archive).await?;

    Ok(ImportStepOutput {
        dataset,
        extra,
    })
}

fn check_files_in_archive(zip_archive: &ZipArchive<&mut File>) -> Result<(), ImportError> {
    let actual_file_names: Vec<&str> = zip_archive.file_names().collect();

    let mut missing_files_to_import = Vec::from(GTFS_FILES_TO_IMPORT);
    missing_files_to_import.retain(|imp_file| !actual_file_names.contains(&imp_file));

    if missing_files_to_import.len() > 0 {
        panic!("Missing a file that we NEED to import"); // TODO: Make this a result error
    }

    let mut missing_required_files = Vec::from(GTFS_REQUIRED_FILES);
    missing_required_files.retain(|req_file| !actual_file_names.contains(&req_file));

    if missing_required_files.len() > 0 {
        todo!("Make rule violations");
    }

    let mut unknown_files = actual_file_names;
    unknown_files.retain(|file| !GTFS_REQUIRED_FILES.contains(&file) && !GTFS_OTHER_FILES.contains(&file));

    // TODO: Make non-critical rule violation if there is a unknown file

    Ok(())
}

async fn import_gtfs_files<'lifetime>(
    zip_archive: &mut ZipArchive<&mut File>
) -> Result<ImportStepExtra, ImportError> {
    let mut tmp_files: HashMap<String, PathBuf> = HashMap::default();

    for filename in GTFS_FILES_TO_IMPORT {
        let mut tmp_file = NamedTempFile::new()?;
        let mut file = zip_archive.by_name(filename)?;
        std::io::copy(&mut file, &mut tmp_file)?;

        tmp_files.insert(
            filename.replace(".txt", ""),
            tmp_file.into_temp_path().keep()?,
        );
    }

    let calendar = LazyCsvReader::new(
        tmp_files.get("calendar").expect("No calendar file found").canonicalize().unwrap().to_str().unwrap()
    ).finish()?
        .select([
            col("service_id").cast(DataType::String),
            col("monday").cast(DataType::Boolean),
            col("tuesday").cast(DataType::Boolean),
            col("wednesday").cast(DataType::Boolean),
            col("thursday").cast(DataType::Boolean),
            col("friday").cast(DataType::Boolean),
            col("saturday").cast(DataType::Boolean),
            col("sunday").cast(DataType::Boolean),
            col("start_date").cast(DataType::String).str().to_date(gtfs_date_format()),
            col("end_date").cast(DataType::String).str().to_date(gtfs_date_format()),
        ]);

    let stop_times = LazyCsvReader::new(
        tmp_files.get("stop_times").expect("No stop_times file found").canonicalize().unwrap().to_str().unwrap()
    ).finish()?
        .select([
            col("trip_id").cast(DataType::String),
            col("stop_id").cast(DataType::String),
            // Cast arrival and departure time to durations, since GTFS spec allows for times that
            // are larger than 24 hours (e.g. 25:42:00). Built-in methods for time of polars would
            //fail in this case. Think of these fields as "duration from midnight".
            col("arrival_time")
                .map(
                    |t| Ok(Some(gtfs_time_to_ms(t)?)),
                    GetOutput::from_type(DataType::Duration(TimeUnit::Milliseconds)),
                )
                .cast(DataType::Duration(TimeUnit::Milliseconds)),
            col("departure_time")
                .map(
                    |t| Ok(Some(gtfs_time_to_ms(t)?)),
                    GetOutput::from_type(DataType::Duration(TimeUnit::Milliseconds)),
                )
                .cast(DataType::Duration(TimeUnit::Milliseconds)),
            // TODO: Convert to duration
            col("stop_sequence").cast(DataType::UInt32),
        ]);

    let stops = LazyCsvReader::new(
        tmp_files.get("stops").expect("No stops file found").canonicalize().unwrap().to_str().unwrap()
    ).finish()?
        .select([
            col("stop_id").cast(DataType::String),
            col("stop_lat").cast(DataType::Float32), // f32 for coordinates might be too little (~2m precision?)
            col("stop_lon").cast(DataType::Float32),
        ]);

    let trips = LazyCsvReader::new(
        tmp_files.get("trips").expect("No trips file found").canonicalize().unwrap().to_str().unwrap()
    ).finish()?
        .select([
            col("route_id").cast(DataType::String),
            col("service_id").cast(DataType::String),
            col("trip_id").cast(DataType::String),
        ]);

    Ok(ImportStepExtra::Gtfs {
        calendar,
        stops,
        trips,
        stop_times,
        temporary_files: tmp_files.into_iter().map(|(_, path)| path).collect(),
    })
}