use polars::datatypes::DataType;
use polars::prelude::{col, GetOutput, LazyCsvReader, LazyFileListReader, Schema, TimeUnit};
use std::collections::HashMap;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;
use zip::ZipArchive;

use crate::gtfs_file::*;
use crate::step1_fetch_data::FetchStepOutput;
use crate::step2_import_data::{ImportError, ImportStepExtra, ImportStepOutput};

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
        return Err(ImportError::MissingFile);
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
    let schema = gtfs_schemas();

    for filename in GTFS_FILES_TO_IMPORT {
        let mut tmp_file = NamedTempFile::new()?;
        let mut file = zip_archive.by_name(filename)?;
        std::io::copy(&mut file, &mut tmp_file)?;

        tmp_files.insert(
            filename.replace(".txt", ""),
            tmp_file.into_temp_path().keep()?,
        );
    }


    let calendar_reader = LazyCsvReader::new(
        tmp_files.get("calendar").expect("No calendar file found").canonicalize()?.to_str().unwrap()
    );

    let mut calendar_schema = calendar_reader.clone().finish()?.collect_schema()?.deref().clone();
    let expected_calendar_schema = Schema::from_iter(schema.calendar.required_fields);
    calendar_schema.merge(expected_calendar_schema);

    let calendar = calendar_reader
        .with_schema(Some(Arc::new(calendar_schema)))
        .finish()?
        .select([
            col("service_id"),
            col("monday").cast(DataType::Boolean),
            col("tuesday").cast(DataType::Boolean),
            col("wednesday").cast(DataType::Boolean),
            col("thursday").cast(DataType::Boolean),
            col("friday").cast(DataType::Boolean),
            col("saturday").cast(DataType::Boolean),
            col("sunday").cast(DataType::Boolean),
            col("start_date").str().to_date(gtfs_date_format()),
            col("end_date").str().to_date(gtfs_date_format()),
        ]);

    
    let stop_times_reader = LazyCsvReader::new(
        tmp_files.get("stop_times").expect("No stop_times file found").canonicalize()?.to_str().unwrap()
    );

    let mut stop_times_schema = stop_times_reader.clone().finish()?.collect_schema()?.deref().clone();
    let expected_stop_times_schema = Schema::from_iter(schema.stop_times.required_fields);
    stop_times_schema.merge(expected_stop_times_schema);

    let stop_times = stop_times_reader
        .with_schema(Some(Arc::new(Schema::from_iter(stop_times_schema))))
        .finish()?
        .select([
            col("trip_id"),
            col("stop_id"),
            // Cast arrival and departure time to durations, since GTFS spec allows for times that
            // are larger than 24 hours (e.g. 25:42:00). Built-in methods for time of polars would
            // fail in this case. Think of these fields as "duration from midnight".
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
            col("stop_sequence"),
        ]);


    let stops_reader = LazyCsvReader::new(
        tmp_files.get("stops").expect("No stop_times file found").canonicalize()?.to_str().unwrap()
    );

    let mut stops_schema = stops_reader.clone().finish()?.collect_schema()?.deref().clone();
    let expected_stops_schema = Schema::from_iter(schema.stops.required_fields);
    stops_schema.merge(expected_stops_schema);

    let stops = stops_reader
        .with_schema(Some(Arc::new(Schema::from_iter(stops_schema))))
        .finish()?
        .select([
            col("stop_id"),
            col("stop_lat"),
            col("stop_lon"),
        ]);


    let trips_reader = LazyCsvReader::new(
        tmp_files.get("trips").expect("No stop_times file found").canonicalize()?.to_str().unwrap()
    );

    let mut trips_schema = trips_reader.clone().finish()?.collect_schema()?.deref().clone();
    let expected_trips_schema = Schema::from_iter(schema.trips.required_fields);
    trips_schema.merge(expected_trips_schema);

    let trips = trips_reader
        .with_schema(Some(Arc::new(Schema::from_iter(trips_schema))))
        .finish()?
        .select([
            col("route_id"),
            col("service_id"),
            col("trip_id"),
        ]);

    Ok(ImportStepExtra::Gtfs {
        calendar,
        stops,
        trips,
        stop_times,
        temporary_files: tmp_files.into_iter().map(|(_, path)| path).collect(),
    })
}