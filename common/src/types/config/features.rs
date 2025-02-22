use either::Either;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FeatureConfig {
    pub preprocessing: PreprocessingConfig
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PreprocessingConfig {
    validation: ValidationConfigOrBool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(transparent)]
struct ValidationConfigOrBool {
    #[serde(with = "either::serde_untagged")]
    inner: Either<bool, ValidationConfig>
}

impl Default for ValidationConfigOrBool {
    fn default() -> Self { 
        Self { inner: Either::Left(true) }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ValidationConfig {
    
}