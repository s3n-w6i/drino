use std::time::SystemTime;

use async_trait::async_trait;
use chrono::{DateTime, Duration};
use futures::future;
use tokio::runtime::Runtime;

use crate::algorithm::{AllRange, PreprocessingInput, PreprocessingResult, PreprocessInit, Range, RoutingAlgorithm};
use crate::direct_connections::DirectConnections;
use crate::raptor::RaptorAlgorithm;
use crate::tp::transfer_patterns::{TransferPatternMap, TransferPatterns};

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

mod transit_network;
mod transfer_patterns;

pub struct TransferPatternsAlgorithm {
    direct_connections: DirectConnections,
    pub transfer_patterns: TransferPatterns,
}

impl RoutingAlgorithm for TransferPatternsAlgorithm {}

#[async_trait]
impl PreprocessInit for TransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<Self> {
        let direct_connections = DirectConnections::try_from(input.clone())?;
        let raptor = RaptorAlgorithm::preprocess(input, direct_connections.clone())?;

        let rt = Runtime::new().unwrap();
        let result = rt.block_on(async {
            let time = SystemTime::now();
            let mut handles = vec![];
            for stop in raptor.clone().stops {
                let raptor = raptor.clone();
                handles.push(tokio::spawn(async move {
                    let res  = raptor.query_range_all(
                        Range {
                            earliest_departure: DateTime::from_timestamp_millis(0).unwrap(),
                            start: stop,
                            range: Duration::weeks(1),
                        }
                    ).await;
                    //dbg!(&res);
                }));
            }
            future::join_all(handles).await;
            //dbg!(&time.elapsed().unwrap());
            //res
        });

        dbg!(result);

        /*let direct_connections = DirectConnections::try_from(input)?;
        let network = TransitNetwork::try_from(direct_connections.clone())?;
        debug!("Network graph stats: {} nodes ({} transfer), {} edges", network.graph.raw_nodes().len(), network.transfer_node_ids.len(), network.graph.raw_edges().len());
        let transfer_patterns = TransferPatterns::from(network);

        Ok(Self {
            direct_connections,
            transfer_patterns,
        })*/

        Ok(Self {
            direct_connections,
            transfer_patterns: TransferPatterns(TransferPatternMap::new()),
        })
    }
}