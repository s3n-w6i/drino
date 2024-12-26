use serde::{Deserialize, Serialize};
use crate::types::dataset::{Dataset, DatasetGroup};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1")]
    Version1 {
        datasets: Vec<Dataset>,
        #[serde(default)]
        dataset_groups: Vec<DatasetGroup>,
    }
}