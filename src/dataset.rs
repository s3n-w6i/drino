use serde::Deserialize;
use crate::config::{DataSource, License};

#[derive(Debug, Deserialize, Clone)]
pub struct DatasetGroup {
    pub name: String,
    #[serde(default = "bool::default")] // false
    pub consistent_ids: bool
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dataset {
    pub id: String,
    pub src: DataSource,
    pub format: DatasetFormat,
    pub license: Option<License>,
    #[serde(default, rename = "groups")]
    pub group_names: Vec<String>,
    // TODO: Fetch interval et al
}

#[derive(Debug, Deserialize, Clone)]
pub enum DatasetFormat {
    #[serde(rename = "gtfs")]
    Gtfs,
    #[serde(rename = "gtfs-rt")]
    GtfsRt,
}