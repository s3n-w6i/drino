use serde::Deserialize;
use std::collections::HashMap;
use url::Url;
use crate::dataset::{Dataset, DatasetGroup};

#[derive(Debug, Deserialize)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1")]
    Version1 {
        datasets: Vec<Dataset>,
        #[serde(default)]
        dataset_groups: Vec<DatasetGroup>,
    }
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