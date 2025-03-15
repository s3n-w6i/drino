use serde::{Deserialize, Serialize};
use crate::types::config::dataset::{Dataset, DatasetGroup};
use crate::types::config::features::FeatureConfig;

pub mod dataset;
pub mod features;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1")]
    Version1 {
        datasets: Vec<Dataset>,
        #[serde(default)]
        dataset_groups: Vec<DatasetGroup>,
        #[serde(default)]
        features: FeatureConfig,
    }
}