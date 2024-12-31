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
    fn preprocess(input: PreprocessingInput, save_to_disk: bool) -> PreprocessingResult<Self> {
        if save_to_disk {
            unimplemented!()
        }

        let direct_connections = DirectConnections::try_from(input.clone())?;
        let raptor = Arc::new(RaptorAlgorithm::preprocess(input.clone(), direct_connections.clone())?);

        let tp_table = Arc::new(Mutex::new(TransferPatternsTable::new()));

        // Also keep a graph representation when in debugging mode. This is useful for checking the
        // validity of what we build.
        #[allow(unused_variables)] // for the regular compiler, where this is not used at all
        let tp_graph = Arc::new(Mutex::new(TransferPatternsGraphs::new(raptor.stop_mapping.0.clone())));

        let total = raptor.num_stops() as u64;
        run_with_pb("preprocessing", "Calculating local transfers in a single cluster", total, false, |pb| {
            raptor.stop_mapping.0.par_iter()
                .map(|stop| {
                    Arc::clone(&raptor).query_range_all(Range {
                        earliest_departure: DateTime::from_timestamp_millis(0).unwrap(),
                        start: *stop,
                        range: Duration::weeks(1),
                    })
                })
                .filter_map(|result| result.ok())
                .map(|range_out| {
                    // Also build the graph version in debug
                    #[cfg(debug_assertions)] {
                        let tp_graph = Arc::clone(&tp_graph);
                        // Add this chunk to our existing transfer patterns graph
                        let mut tp_graph = tp_graph.lock().unwrap();
                        tp_graph.add(range_out.clone());
                        drop(tp_graph);
                    }

                    // Add the collected results to the table of transfer patterns
                    let tp_table = Arc::clone(&tp_table);
                    let mut tp_table = tp_table.lock().unwrap();
                    let res = tp_table.add(range_out);
                    drop(tp_table);

                    res
                })
                .for_each(|_| {
                    pb.inc(1);
                });
        });


        #[cfg(debug_assertions)] {
            let tp_graph = Arc::try_unwrap(tp_graph)
                .expect("Lock is still owned by others").into_inner().unwrap();
            // Check that graphs are acyclic. Expensive to compute, so only do that in debug.
            tp_graph.validate();
        }

        let tp_table = Arc::try_unwrap(tp_table)
            .expect("Lock is still owned by others").into_inner().unwrap();

        Ok(Self {
            direct_connections,
            transfer_patterns: tp_table,
        })
    }
}