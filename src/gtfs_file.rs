use polars::prelude::Field;

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

#[derive(Debug)]
pub struct GtfsFile {
    pub name: &'static str,
    pub required_fields: Vec<Field>
}

pub fn gtfs_schemas() -> [GtfsFile; 5] {[
    GtfsFile {
        name: "agency",
        required_fields: vec![],
    },
    GtfsFile {
        name: "routes",
        required_fields: vec![],
    },
    GtfsFile {
        name: "stop_times",
        required_fields: vec![],
    },
    GtfsFile {
        name: "stops",
        required_fields: vec![],
    },
    GtfsFile {
        name: "trips",
        required_fields: vec![],
    }
]}