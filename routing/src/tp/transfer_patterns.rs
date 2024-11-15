use crate::algorithm::RangeOutput;
use crate::journey::Journey;
use common::types::StopId;
use itertools::Itertools;
use petgraph::algo::is_cyclic_directed;
use petgraph::data::DataMap;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph, Incoming};
use polars::datatypes::DataType;
use polars::error::PolarsResult;
use polars::frame::DataFrame;
use polars::prelude::Column;
use std::fmt::Debug;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

#[derive(Debug)]
enum NodeType {
    Target,
    Prefix,
    Root,
}

#[derive(Debug)]
pub struct TransferPatternsGraphs {
    dags: Vec<Graph<(StopId, NodeType), (), Directed>>,
}

impl TransferPatternsGraphs {
    pub(crate) fn new(num_stops: usize) -> Self {
        // Build a graph for each origin stop
        let dags = (0..num_stops)
            .map(|root_node_idx| {
                // Initialize a graph, where the root node is already present
                let mut graph = Graph::with_capacity(num_stops, num_stops);
                graph.add_node((StopId(root_node_idx as u32), NodeType::Root));
                graph
            })
            .collect_vec();

        Self { dags }
    }

    pub(crate) fn add(&mut self, results: Vec<RangeOutput>) {
        let all_journeys = results.into_iter()
            .flat_map(|res| { res.journeys });

        for journey in all_journeys {
            self.add_journey(journey);
        }
    }

    fn add_journey(&mut self, journey: Journey) {
        let journey_start_idx = journey.departure_stop().0 as usize;
        let journey_end = *journey.arrival_stop();
        // Find the graph to which we want to add this journey
        let graph = &mut self.dags[journey_start_idx];

        // At least the root node is always in the graph (see Self::new())
        debug_assert!(graph.node_count() > 0);

        // The node with index 0 is always the root node, from which all transfer patterns go
        // out from. So, start here, because trip starts here as well
        let mut current_node_idx = NodeIndex::from(0);
        debug_assert!(&graph.node_weight(current_node_idx).unwrap().0 == journey.departure_stop());

        for leg in journey.legs() {
            let end = leg.end();
            let start = leg.start();
            let last_leg = end == &journey_end;

            debug_assert!(
                start == &graph.node_weight(current_node_idx).unwrap().0,
                "Expected start of leg ({start}) to match the current_node's ({})",
                &graph.node_weight(current_node_idx).unwrap().0
            );

            // Distinguish between the last leg (target station nodes) and intermediate legs 
            // (prefix nodes):
            // - prefix nodes may occur multiple time in the graph
            // - target station nodes must only occur once, in order to be able to build the
            //   query graph more efficiently
            match last_leg {
                // Insert prefix node
                false => {
                    // Find a node on the path we've gone whose stop ID is the end of this leg.
                    // Direction is `Incoming`, since edges are in the opposite direction of
                    // travel.
                    let end_node_idx = graph.neighbors_directed(current_node_idx, Incoming)
                        .find(|n| {
                            let (stop_id, node_type) = graph.node_weight(*n).unwrap();
                            stop_id == end && matches!(node_type, NodeType::Prefix)
                        });

                    match end_node_idx {
                        None => {
                            let end_node_idx = graph.add_node((*end, NodeType::Prefix));

                            // Add an edge from the end to start. This is reversed, so that on query time, we
                            // can start at our target station and then find a way to the origin station
                            // efficiently.
                            let start_node_idx = current_node_idx;
                            graph.add_edge(end_node_idx, start_node_idx, ());

                            current_node_idx = end_node_idx;
                        }
                        Some(end_node_idx) => {
                            current_node_idx = end_node_idx;
                        }
                    }
                }
                // Insert target station node
                true => {
                    let target = end;
                    // TODO: This is O(n). Make it more efficient if possible.
                    let candidate_node_idx = graph.node_indices()
                        .find(|n| graph.node_weight(*n).unwrap().0 == *target);

                    fn add_new_target_node(graph: &mut Graph<(StopId, NodeType), ()>, target_stop: StopId, from: NodeIndex) {
                        let target_node_idx = graph.add_node((target_stop, NodeType::Target));
                        graph.add_edge(target_node_idx, from, ());
                    }

                    if let Some(target_node_idx) = candidate_node_idx {
                        // A node with the same value already exists. It might be a prefix node,
                        // in which case we would need to create a new node.
                        if matches!(graph.node_weight(target_node_idx).unwrap().1, NodeType::Target) {
                            // The target node already exists, and it is valid (not a prefix node).
                            // Just add an edge (again, in reverse)
                            if !graph.contains_edge(target_node_idx, current_node_idx) {
                                graph.add_edge(target_node_idx, current_node_idx, ());
                            }
                        } else {
                            add_new_target_node(graph, *target, current_node_idx);
                        }
                        // No need to set next current_node_idx
                    } else {
                        // There is no target node yet
                        // Create one and an edge to connect to it
                        add_new_target_node(graph, *target, current_node_idx);
                    }
                }
            }
        }
    }

    pub(crate) fn rename_stops(&mut self, from: &Vec<StopId>, to: &Vec<StopId>) {
        // todo!()
    }

    pub(crate) fn print(&self, stop_id: StopId) {
        let graph = &self.dags[stop_id.0 as usize];

        println!(
            "{:?}",
            Dot::with_attr_getters(
                graph,
                &[Config::EdgeNoLabel],
                &|_graph, _e| { "".into() },
                &|_graph, (_n, (_s, t))| {
                    match t {
                        NodeType::Root => { "shape = diamond width = 4 height = 2".into() }
                        NodeType::Target => { "shape = square width = 1 height = 1".into() }
                        NodeType::Prefix => { "shape = circle width = 1 height = 1".into() }
                    }
                },
            )
        );
    }

    #[cfg(debug_assertions)]
    pub(crate) fn validate(&self) {
        for graph in &self.dags {
            // Validate acyclic property
            debug_assert!(
                !is_cyclic_directed::<&Graph<(StopId, NodeType), (), Directed>>(graph),
                "Every transfer pattern graph must be acyclic."
            );
            
            // Check that there is only one target node per stop
            let duplicate_targets = graph.node_weights()
                // Only check uniqueness of target nodes (there can be multiple prefix nodes per stop)
                .filter(|(_stop, node_type)| matches!(node_type, NodeType::Target) )
                .duplicates_by(|(stop, _t)| stop)
                .collect_vec();
            debug_assert!(
                duplicate_targets.is_empty(),
                "There must only be one target node for each stop. These are duplicates: {:?}",
                duplicate_targets.iter().map(|(stop, _)| stop).collect_vec()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::journey::Journey;
    use crate::journey::Leg::Ride;
    use crate::tp::transfer_patterns::TransferPatternsGraphs;
    use chrono::DateTime;
    use common::types::{StopId, TripId};

    #[test]
    fn test_tp_adding() {
        // This the same example as what's in the transfer patterns paper in Fig. 1
        // A corresponds to Stop ID 0, B to 1 and so on...
        let mut tp = TransferPatternsGraphs::new(5);

        let a = StopId(0);
        let b = StopId(1);
        let c = StopId(2);
        let d = StopId(3);
        let e = StopId(4);

        let ab = Ride {
            trip: TripId(41),
            boarding_stop: a,
            alight_stop: b,
            boarding_time: DateTime::UNIX_EPOCH,
            alight_time: DateTime::UNIX_EPOCH,
        };
        let bc = Ride {
            trip: TripId(43),
            boarding_stop: b,
            alight_stop: c,
            boarding_time: DateTime::UNIX_EPOCH,
            alight_time: DateTime::UNIX_EPOCH,
        };
        let de = Ride {
            trip: TripId(45),
            boarding_stop: d,
            alight_stop: e,
            boarding_time: DateTime::UNIX_EPOCH,
            alight_time: DateTime::UNIX_EPOCH,
        };

        // A -> E
        tp.add_journey(Journey::from(vec![
            Ride {
                trip: TripId(42),
                boarding_stop: a,
                alight_stop: e,
                boarding_time: DateTime::UNIX_EPOCH,
                alight_time: DateTime::UNIX_EPOCH,
            }
        ]));

        // A -> B -> E
        tp.add_journey(Journey::from(vec![
            ab.clone(),
            Ride {
                trip: TripId(42),
                boarding_stop: b,
                alight_stop: e,
                boarding_time: DateTime::UNIX_EPOCH,
                alight_time: DateTime::UNIX_EPOCH,
            }
        ]));

        // A -> B -> C
        tp.add_journey(Journey::from(vec![
            ab.clone(),
            bc.clone()
        ]));

        // A -> B -> D -> E
        tp.add_journey(Journey::from(vec![
            ab.clone(),
            Ride {
                trip: TripId(44),
                boarding_stop: b,
                alight_stop: d,
                boarding_time: DateTime::UNIX_EPOCH,
                alight_time: DateTime::UNIX_EPOCH,
            },
            de.clone(),
        ]));

        // A -> B -> C -> D -> E
        tp.add_journey(Journey::from(vec![
            ab.clone(),
            bc.clone(),
            Ride {
                trip: TripId(31),
                boarding_stop: c,
                alight_stop: d,
                boarding_time: DateTime::UNIX_EPOCH,
                alight_time: DateTime::UNIX_EPOCH,
            },
            de.clone()
        ]));

        tp.print(a);
        todo!("assertions");
    }

    #[test]
    fn test_tp_double_insert() {
        let mut tp = TransferPatternsGraphs::new(2);

        for _ in 0..2 {
            tp.add_journey(Journey::from(vec![
                Ride {
                    trip: TripId(0),
                    boarding_stop: StopId(0),
                    alight_stop: StopId(1),
                    boarding_time: DateTime::UNIX_EPOCH,
                    alight_time: DateTime::UNIX_EPOCH
                }
            ]));
        }

        tp.print(StopId(0));

        todo!("assertions");
    }
}

#[derive(Debug)]
pub struct TransferPatternsTable(pub(crate) DataFrame);

impl TransferPatternsTable {
    pub(crate) fn new() -> PolarsResult<Self> {
        let start_series = Column::new_empty("start".into(), &DataType::UInt32);
        let end_series = Column::new_empty("end".into(), &DataType::UInt32);

        Ok(Self(DataFrame::new(vec![start_series, end_series])?))
    }
}

/*TODO: impl TryFrom<TransferPatternsDAGs> for TransferPatternsTable {
    type Error = PolarsError;

    fn try_from(TransferPatternsDAGs(graph): TransferPatternsDAGs) -> Result<Self, Self::Error> {
        let mut table = Self::new()?;
        
        let rows = graph.all_edges()
            .map(|(a, b, ..)| {
                Row::new(vec![a.into(), b.into()])
            })
            .collect_vec();
        
        if !rows.is_empty() {
            table.0.vstack_mut_unchecked(
                &DataFrame::from_rows(&rows)?
            );

            table.0.align_chunks();
        }
        
        Ok(table)
    }
}*/