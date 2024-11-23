use crate::algorithm::{AllRange, PreprocessInit, PreprocessingInput, PreprocessingResult, Range};
use crate::direct_connections::DirectConnections;
use crate::raptor::RaptorAlgorithm;
use crate::tp::transfer_pattern_ds::graph::TransferPatternsGraphs;
use crate::tp::transfer_pattern_ds::table::TransferPatternsTable;
use crate::tp::TransferPatternsAlgorithm;
use async_trait::async_trait;
use chrono::{DateTime, Duration};
use common::util::logging::run_with_pb;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::{Arc, Mutex};

#[async_trait]
impl PreprocessInit for TransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<Self> {
        let direct_connections = DirectConnections::try_from(input.clone())?;
        let raptor = Arc::new(RaptorAlgorithm::preprocess(input.clone(), direct_connections.clone())?);

        let tp_table = TransferPatternsTable::new()?;

        // Also keep a graph representation when in debugging mode. This is useful for checking the
        // validity of what we build.
        #[allow(unused_assignments)] // for the regular compiler, where this is not used at all        
        let tp_graph = if cfg!(debug_assertions) {
            &Some(Arc::new(Mutex::new(
                TransferPatternsGraphs::new(raptor.stops.len())
            )))
        } else { &None };

        let total = raptor.stops.len() as u64;
        run_with_pb("preprocessing", "Calculating local transfers in a single cluster", total, false, |pb| {
            raptor.stops.par_iter()
                .map(|stop| {
                    Arc::clone(&raptor).query_range_all(Range {
                        earliest_departure: DateTime::from_timestamp_millis(0).unwrap(),
                        start: *stop,
                        range: Duration::weeks(1),
                    })
                })
                .filter_map(|result| result.ok())
                .map(move |range_out| {
                    // Add the collected results to the table of transfer patterns
                    // TODO

                    // Also build the graph version in debug
                    #[cfg(debug_assertions)] {
                        tp_graph.as_ref().map(|tp_graph| {
                            let tp_graph = Arc::clone(tp_graph);
                            // Add this chunk to our existing transfer patterns graph
                            let mut tp_graph = tp_graph.lock().unwrap();
                            tp_graph.add(vec![range_out]);
                            drop(tp_graph);
                        });
                    }
                })
                .for_each(|_| pb.inc(1));
        });


        #[cfg(debug_assertions)] {
            tp_graph.to_owned().map(|tp_graph| {
                let tp_graph = Arc::try_unwrap(tp_graph)
                        .expect("Lock is still owned by others").into_inner().unwrap();
                // Check that graphs are acyclic. Expensive to compute, so only do that in debug.
                tp_graph.validate();
            });
        }

        Ok(Self {
            direct_connections,
            transfer_patterns: tp_table,
        })
    }
}