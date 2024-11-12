use serde::Deserialize;
use crate::types::dataset::{Dataset, DatasetGroup};

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