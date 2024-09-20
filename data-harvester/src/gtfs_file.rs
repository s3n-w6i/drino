use polars::datatypes::{AnyValue, DataType};
use polars::error::{ErrString, PolarsError};
use polars::export::chrono::NaiveDateTime;
use polars::prelude::{Field, StrptimeOptions};
use polars::series::Series;

pub const GTFS_REQUIRED_FILES: [&str; 5] = [
    "agency.txt", "stops.txt", "routes.txt", "trips.txt", "stop_times.txt"
];
pub const GTFS_OTHER_FILES: [&str; 21] = [
    "calendar.txt",
    "calendar_dates.txt",
    "fare_attributes.txt",
    "fare_rules.txt",
    "timeframes.txt",
    "fare_media.txt",
    "fare_products.txt",
    "fare_leg_rules.txt",
    "fare_transfer_rules.txt",
    "areas.txt",
    "stop_areas.txt",
    "networks.txt",
    "route_networks.txt",
    "shapes.txt",
    "frequencies.txt",
    "transfers.txt",
    "pathways.txt",
    "levels.txt",
    "translations.txt",
    "feed_info.txt",
    "attributions.txt",
];
pub const GTFS_FILES_TO_IMPORT: [&str; 4] = [
    "calendar.txt",
    "stops.txt",
    "trips.txt",
    "stop_times.txt"
];

pub fn gtfs_date_format() -> StrptimeOptions {
    StrptimeOptions {
        format: Some("%Y%m%d".into()),
        strict: true,
        exact: true,
        cache: true,
    }
}

#[inline]
pub fn gtfs_time_to_ms(times: Series) -> Result<Series, PolarsError> {
    let strings = times.rechunk().iter()
        .map(|time| {
            match time {
                AnyValue::String(time) => {
                    // replace 03:42:31 with (3 * (60 * 60)) + (42 * 60) + (31 * 1)
                    let result: u32 = time.split(":")
                        .zip([60 * 60, 60, 1]) // seconds in hours, minutes, seconds
                        .map(|(factor, seconds_in_unit)| {
                            let factor = factor.parse::<u32>();
                            return if let Ok(factor) = factor {
                                Ok(factor * seconds_in_unit)
                            } else {
                                Err(PolarsError::ComputeError(ErrString::from("Could not parse duration")))
                            }
                        })
                        .collect::<Result<Vec<u32>, PolarsError>>()?.into_iter()
                        .sum::<u32>() * 1000;
                    Ok(result)
                }
                _ => Err(PolarsError::SchemaMismatch(ErrString::from("Expected string")))
            }
        })
        .collect::<Result<Vec<u32>, PolarsError>>()?;
    let series: Series = strings.into_iter().collect();
    Ok(series)
}

#[derive(Debug)]
pub struct GtfsFile {
    pub name: &'static str,
    pub required_fields: Vec<Field>,
}

pub struct GtfsDataset {
    pub agency: GtfsFile,
    pub calendar: GtfsFile,
    pub routes: GtfsFile,
    pub stop_times: GtfsFile,
    pub stops: GtfsFile,
    pub trips: GtfsFile,
}

pub fn gtfs_schemas() -> GtfsDataset {
    GtfsDataset {
        agency: GtfsFile {
            name: "agency",
            required_fields: vec![
                Field { name: "agency_id".into(), dtype: DataType::String },
                Field { name: "agency_timezone".into(), dtype: DataType::String },
            ],
        },
        calendar: GtfsFile {
            name: "calendar",
            required_fields: vec![
                Field { name: "service_id".into(), dtype: DataType::String },
                Field { name: "monday".into(), dtype: DataType::UInt32 },
                Field { name: "tuesday".into(), dtype: DataType::UInt32 },
                Field { name: "wednesday".into(), dtype: DataType::UInt32 },
                Field { name: "thursday".into(), dtype: DataType::UInt32 },
                Field { name: "friday".into(), dtype: DataType::UInt32 },
                Field { name: "saturday".into(), dtype: DataType::UInt32 },
                Field { name: "sunday".into(), dtype: DataType::UInt32 },
                Field { name: "start_date".into(), dtype: DataType::String },
                Field { name: "end_date".into(), dtype: DataType::String },
            ],
        },
        routes: GtfsFile {
            name: "routes",
            required_fields: vec![
                Field { name: "route_id".into(), dtype: DataType::String },
                Field { name: "agency_id".into(), dtype: DataType::String },
            ],
        },
        stop_times: GtfsFile {
            name: "stop_times",
            required_fields: vec![
                Field { name: "trip_id".into(), dtype: DataType::String },
                Field { name: "arrival_time".into(), dtype: DataType::String },
                Field { name: "departure_time".into(), dtype: DataType::String },
                Field { name: "stop_sequence".into(), dtype: DataType::UInt32 },
            ],
        },
        stops: GtfsFile {
            name: "stops",
            required_fields: vec![
                Field { name: "stop_id".into(), dtype: DataType::String },
                Field { name: "stop_lat".into(), dtype: DataType::Float32 }, // f32 for coordinates might be too little (~2m precision?)
                Field { name: "stop_lon".into(), dtype: DataType::Float32 }, // f32 for coordinates might be too little (~2m precision?)
            ],
        },
        trips: GtfsFile {
            name: "trips",
            required_fields: vec![
                Field { name: "route_id".into(), dtype: DataType::String },
                Field { name: "service_id".into(), dtype: DataType::String },
                Field { name: "trip_id".into(), dtype: DataType::String },
            ],
        },
    }
}