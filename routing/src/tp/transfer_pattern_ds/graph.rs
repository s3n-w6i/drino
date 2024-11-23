use crate::algorithm::RangeOutput;
use crate::journey::Journey;
use common::types::StopId;
use itertools::Itertools;
use petgraph::algo::is_cyclic_directed;
use petgraph::data::DataMap;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::{Directed, Graph, Incoming};
use std::fmt::Debug;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf
/// This graph only exists for debugging purposes. In the real world, we use the transfer pattern
/// table to make easier to store them. Be careful: Stops are not mapped to their real ids here!

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
enum NodeType {
    Target,
    Prefix,
    Root,
}

/// This is just one of the graphs with a single root node
/// It displays all the patterns of journeys that start at that root node
type TpGraph = Graph<(StopId, NodeType), (), Directed>;

#[derive(Debug)]
pub struct TransferPatternsGraphs {
    dags: Vec<TpGraph>,
}

fn format_graph<'a>(graph: &TpGraph) -> Dot<'a, &TpGraph> {
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
                        .find(|n| {
                            let (stop_id, node_type) = graph.node_weight(*n).unwrap();
                            stop_id == target && matches!(node_type, NodeType::Target)
                        });

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

    pub(crate) fn format_as_dot<'a>(&self, stop_id: StopId) -> Dot<'a, &TpGraph> {
        let graph = &self.dags[stop_id.0 as usize];

        format_graph(graph)
    }
    
    pub(crate) fn print(&self, stop_id: StopId) {
        println!("{:?}", self.format_as_dot(stop_id));
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
                .filter(|(_stop, node_type)| matches!(node_type, NodeType::Target))
                // Find the duplicate stops
                .duplicates_by(|(stop, _t)| stop)
                .collect_vec();
            debug_assert!(
                duplicate_targets.is_empty(),
                "There must only be one target node for each stop. These are duplicates: {:?}. Graph: {:?}",
                duplicate_targets.iter().map(|(stop, _)| stop).collect_vec(),
                format_graph(graph)
            );
        }
    }

    pub(self) fn nodes(&self, root: StopId) -> Option<impl Iterator<Item=&(StopId, NodeType)> + Sized> {
        self.dags.get::<usize>(root.0 as usize)
            .map(|dag: &Graph<(StopId, NodeType), (), Directed>| {
                dag.node_weights()
            })
    }

    pub(self) fn edges(&self, root: StopId) -> Option<impl Iterator<Item=(&(StopId, NodeType), &(StopId, NodeType))> + Sized + use<'_>> {
        self.dags.get::<usize>(root.0 as usize)
            .map(|dag: &Graph<(StopId, NodeType), (), Directed>| {
                dag.edge_indices()
                    .map(|i| {
                        dag.edge_endpoints(i).unwrap()
                    })
                    .map(|(start_idx, end_idx)| {
                        (dag.node_weight(start_idx).unwrap(), dag.node_weight(end_idx).unwrap())
                    })
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::journey::Journey;
    use crate::journey::Leg::Ride;
    use super::{NodeType, TransferPatternsGraphs};
    use chrono::{DateTime, TimeDelta};
    use common::types::{StopId, TripId};
    use itertools::{assert_equal, Itertools};

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
            alight_time: DateTime::UNIX_EPOCH + TimeDelta::seconds(1),
        };
        let bc = Ride {
            trip: TripId(43),
            boarding_stop: b,
            alight_stop: c,
            boarding_time: DateTime::UNIX_EPOCH,
            alight_time: DateTime::UNIX_EPOCH + TimeDelta::days(1),
        };
        let de = Ride {
            trip: TripId(45),
            boarding_stop: d,
            alight_stop: e,
            boarding_time: DateTime::UNIX_EPOCH,
            alight_time: DateTime::UNIX_EPOCH + TimeDelta::milliseconds(1),
        };

        // A -> E
        tp.add_journey(Journey::from(vec![
            Ride {
                trip: TripId(42),
                boarding_stop: a,
                alight_stop: e,
                boarding_time: DateTime::UNIX_EPOCH,
                alight_time: DateTime::UNIX_EPOCH + TimeDelta::hours(20),
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
                alight_time: DateTime::UNIX_EPOCH + TimeDelta::hours(1),
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
                alight_time: DateTime::UNIX_EPOCH + TimeDelta::hours(2),
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
                alight_time: DateTime::UNIX_EPOCH + TimeDelta::hours(42),
            },
            de.clone()
        ]));

        tp.print(a);
        todo!("assertions");
    }

    #[test]
    fn test_tp_double_insert() {
        let mut tp = TransferPatternsGraphs::new(3);

        // Do twice
        for _ in 0..2 {
            tp.add_journey(Journey::from(vec![
                Ride {
                    trip: TripId(0),
                    boarding_stop: StopId(0),
                    alight_stop: StopId(1),
                    boarding_time: DateTime::UNIX_EPOCH,
                    alight_time: DateTime::UNIX_EPOCH + TimeDelta::seconds(42),
                }
            ]));

            tp.add_journey(Journey::from(vec![
                Ride {
                    trip: TripId(0),
                    boarding_stop: StopId(0),
                    alight_stop: StopId(1),
                    boarding_time: DateTime::UNIX_EPOCH,
                    alight_time: DateTime::UNIX_EPOCH + TimeDelta::seconds(42),
                },
                Ride {
                    trip: TripId(2),
                    boarding_stop: StopId(1),
                    alight_stop: StopId(2),
                    boarding_time: DateTime::UNIX_EPOCH,
                    alight_time: DateTime::UNIX_EPOCH + TimeDelta::seconds(1),
                }
            ]));
        }

        tp.print(StopId(0));

        tp.validate();

        let expected_nodes = [
            &(StopId(2), NodeType::Target),
            &(StopId(1), NodeType::Target),
            &(StopId(1), NodeType::Prefix),
            &(StopId(0), NodeType::Root),
        ];
        assert_equal(
            tp.nodes(StopId(0)).unwrap().into_iter().sorted(),
            expected_nodes.into_iter().sorted(),
        );
        
        let expected_edges = [
            (&(StopId(2), NodeType::Target), &(StopId(1), NodeType::Prefix)),
            (&(StopId(1), NodeType::Prefix), &(StopId(0), NodeType::Root)),
            (&(StopId(1), NodeType::Target), &(StopId(0), NodeType::Root)),
        ];
        assert_equal(
            tp.edges(StopId(0)).unwrap().into_iter().sorted(),
            expected_edges.into_iter().sorted()
        );
    }
}

