use crate::util::distance::{Distance, Radius};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatasetGroup {
    pub id: String,
    pub consistency: DatasetConsistency
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatasetConsistency {
    #[serde(default)]
    stop_ids: IdConsistency,
    #[serde(default)]
    stop_coordinates: GeoPointConsistency,
    #[serde(default)]
    trip_ids: IdConsistency,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum IdConsistency {
    Fully(bool),
    Partially { tolerance: f32 },
}

impl Default for IdConsistency {
    fn default() -> Self {
        IdConsistency::Partially { tolerance: 0.5 }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(
    untagged,
    expecting = "Invalid or missing consistency definition. Specify either a hard cutoff radius with `radius: 42m` or an attenuation with `equality_radius:` and `inequality_radius:`"
)]
pub enum GeoPointConsistency {
    Attenuation {
        equality_radius: Radius,
        inequality_radius: Radius
    },
    HardCutoff {
        radius: Radius
    }
}

impl Default for GeoPointConsistency {
    fn default() -> Self {
        GeoPointConsistency::HardCutoff { radius: Distance(20.0) }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Dataset {
    pub id: String,
    pub src: DataSource,
    pub format: DatasetFormat,
    pub license: Option<License>,
    #[serde(default, rename = "groups")]
    pub group_ids: Vec<String>,
    // TODO: Fetch interval et al
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DatasetFormat {
    #[serde(rename = "gtfs")]
    Gtfs,
    #[serde(rename = "gtfs-rt")]
    GtfsRt,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(
    untagged,
    expecting = "Invalid or missing data source. Specify either a remote source with `url:` and `headers:` or a local path with `file:` under `src:` of this dataset")
]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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