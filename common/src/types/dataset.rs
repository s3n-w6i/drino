use std::collections::HashMap;
use url::Url;
use serde::Deserialize;

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

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum DataSource {
    URL {
        url: Url,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    File {
        path: String
    }
}

// Identifiers: https://spdx.org/licenses/
#[derive(Debug, Deserialize, Clone)]
pub enum License {
    Custom { src: DataSource },
    #[serde(rename = " CC0-1.0")]
    Cc0_1_0,
    #[serde(rename = "CC-BY-4.0")]
    CcBy4_0,
    #[serde(rename = "CC-BY-NC-4.0")]
    CcByNc4_0,
    #[serde(rename = "CC-BY-ND-4.0")]
    CcByNd4_0,
    #[serde(rename = "CC-BY-SA-4.0")]
    CcBySa4_0,
    #[serde(rename = "CC-BY-NC-ND-4.0")]
    CcByNcNd4_0,
    #[serde(rename = "CC-BY-NC-SA-4.0")]
    CcByNcSa4_0,
    #[serde(rename = "DL-DE-BY-2.0")]
    DlDeBy2_0,
    #[serde(rename = "DL-DE-ZERO-2.0")]
    DlDeZero2_0,
}