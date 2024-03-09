use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use polars::prelude::{LazyCsvReader, LazyFileListReader};
use crate::gtfs_file::{GTFS_FILES_TO_IMPORT, GTFS_OTHER_FILES, GTFS_REQUIRED_FILES};
use tempfile::NamedTempFile;
use zip::ZipArchive;
use crate::preprocessing_steps::step1_fetch_data::FetchStepOutput;
use crate::preprocessing_steps::step2_import_data::{ImportError, ImportStepExtra, ImportStepOutput};

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
        extra
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
    mut zip_archive: &mut ZipArchive<&mut File>
) -> Result<ImportStepExtra, ImportError> {
    let mut tmp_files: HashMap<String, PathBuf> = HashMap::default();

    for filename in GTFS_FILES_TO_IMPORT {
        let mut tmp_file = NamedTempFile::new()?;
        let mut file = zip_archive.by_name(filename)?;
        std::io::copy(&mut file, &mut tmp_file)?;

        tmp_files.insert(
            filename.replace(".txt", ""),
            tmp_file.into_temp_path().keep()?
        );
    }

    let calendar = LazyCsvReader::new(
        tmp_files.get("calendar").expect("No calendar file found").canonicalize().unwrap().to_str().unwrap()
    ).with_infer_schema_length(Some(1_000_000))
        .finish()?;

    let stops = LazyCsvReader::new(
        tmp_files.get("stops").expect("No stops file found").canonicalize().unwrap().to_str().unwrap()
    ).with_infer_schema_length(Some(1_000_000))
        .finish()?;

    let trips = LazyCsvReader::new(
        tmp_files.get("trips").expect("No trips file found").canonicalize().unwrap().to_str().unwrap()
    ).with_infer_schema_length(Some(1_000_000))
        .finish()?;

    let stop_times = LazyCsvReader::new(
        tmp_files.get("stop_times").expect("No stop_times file found").canonicalize().unwrap().to_str().unwrap()
    ).with_infer_schema_length(Some(1_000_000))
        .finish()?;

    Ok(ImportStepExtra::Gtfs {
        calendar,
        stops,
        trips,
        stop_times,
        temporary_files: tmp_files.into_iter().map(|(_, path)| path).collect()
    })
}