extern crate drino_visualization;

use common::types::config::Config;
use common::util::logging;
use log::{info, LevelFilter};
use std::str::FromStr;
use common::types::dataset::{DataSource, Dataset, DatasetConsistency, DatasetFormat, DatasetGroup, GeoPointConsistency, IdConsistency, License};
use url::Url;
use common::util::distance::Distance;
use drino_visualization::run_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::init(LevelFilter::Info);

    run_server(
        // TODO
        Config::Version1 {
            datasets: vec![
                Dataset {
                    id: "dataset-1".into(),
                    format: DatasetFormat::Gtfs,
                    group_ids: vec![ "group-a".into() ],
                    license: Some(License::Cc0_1_0),
                    src: DataSource::URL { url: Url::from_str("https://asdf.com").unwrap(), headers: Default::default() }
                },
                Dataset {
                    id: "dataset-2".into(),
                    format: DatasetFormat::Gtfs,
                    group_ids: vec![ "group-a".into(), "group-b".into() ],
                    license: Some(License::Cc0_1_0),
                    src: DataSource::URL { url: Url::from_str("https://asdf.com").unwrap(), headers: Default::default() }
                },
                Dataset {
                    id: "dataset-3".into(),
                    format: DatasetFormat::GtfsRt,
                    group_ids: vec![ "group-b".into() ],
                    license: Some(License::Cc0_1_0),
                    src: DataSource::URL { url: Url::from_str("https://asdf.com").unwrap(), headers: Default::default() }
                },
            ],
            dataset_groups: vec![
                DatasetGroup {
                    id: "group-a".into(),
                    consistency: DatasetConsistency {
                        trip_ids: IdConsistency::Fully(true),
                        stop_ids: IdConsistency::Fully(true),
                        stop_coordinates: GeoPointConsistency::HardCutoff {
                            radius: Distance(10f32)
                        }
                    }
                }
            ],
        },
    )
    .await?;

    info!("Server shut down");

    Ok(())
}