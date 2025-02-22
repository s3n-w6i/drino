use std::fmt;
use std::fmt::Display;
use std::io::{Cursor};
use std::time::{SystemTime, UNIX_EPOCH};
use common::types::config::dataset::{Dataset, DataSource};
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

pub async fn fetch_dataset(
    dataset: Dataset
) -> Result<FetchStepOutput, FetchError> {
    match dataset.clone().src {
        DataSource::URL { url, .. } => {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let path_str = format!("./data/datasets/{}/imports/{}", dataset.id, timestamp);
            let path = Path::new(&path_str);
            create_dir_all(path.parent().unwrap())?;
            let mut file = File::create(&path)?;

            let response = reqwest::get(url.clone()).await?;
            let mut content = Cursor::new(response.bytes().await?);
            std::io::copy(&mut content, &mut file)?;

            Ok(FetchStepOutput {
                dataset,
                path: path.to_path_buf(),
            })
        },
        DataSource::File { path } => {
            Ok(FetchStepOutput {
                dataset,
                path: PathBuf::from(path),
            })
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    Reqwest(#[from] reqwest::Error),
    File(#[from] std::io::Error)
}

impl Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            FetchError::Reqwest(err) => err,
            FetchError::File(err) => err
        };
        write!(f, "{}", err)
    }
}



pub struct FetchStepOutput {
    pub(crate) dataset: Dataset,
    pub(crate) path: PathBuf
}