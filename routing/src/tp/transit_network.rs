use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::ops::Add;
use std::time::Duration;

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use polars::export::arrow::array::{Int64Array, UInt32Array};
use polars::prelude::{col, IntoLazy, SortMultipleOptions, CompatLevel};

use crate::algorithm::PreprocessingError;
use crate::direct_connections::DirectConnections;
use crate::tp::transit_network::Edge::{Alight, Board, Ride, StayOn, Waiting};
use crate::tp::transit_network::Node::{Arrival, Departure, Transfer};

const MIN_TRANSFER_TIME_SECS: u64 = 2 * 60;

type TransitNetworkGraph = Graph<Node, Edge>;

#[derive(Debug)]
pub enum Node {
    Transfer { time: Duration, stop_id: u32 },
    Departure { departure_time: Duration, stop_id: u32 },
    Arrival { arrival_time: Duration, stop_id: u32 },
}

impl Node {
    fn time(&self) -> &Duration {
        match self {
            Transfer { time, .. } => time,
            Departure { departure_time, .. } => departure_time,
            Arrival { arrival_time, .. } => arrival_time,
        }
    }
    pub fn stop_id(&self) -> &u32 {
        match self {
            Transfer { stop_id, .. } | Departure { stop_id, .. } | Arrival { stop_id, .. } => stop_id,
        }
    }
}

#[derive(Debug)]
pub enum Edge {
    // Take a journey from one station to the next: From departure node to arrival node
    Ride { line_id: u32, duration: Duration },
    // Continue journey on that trip: From arrival node to departure node
    StayOn { duration: Duration },
    // From transfer node to transfer node
    Waiting { duration: Duration },
    // From arrival node to transfer node
    Alight { duration: Duration },
    // From transfer node to departure node
    Board,
}

impl Edge {
    pub fn cost(&self) -> Cost {
        let duration = match self {
            Ride { duration, .. } |
            StayOn { duration, .. } |
            Waiting { duration, .. } |
            Alight { duration, .. } => *duration,
            Board => Duration::from_secs(0),
        };

        let penalty = match self {
            Ride { .. } => 0.05,
            StayOn { .. } | Waiting { .. } | Board { .. } => 0.0,
            Alight { .. } => 1.0,
        };

        Cost { duration, penalty }
    }
}

pub struct Cost {
    pub penalty: f32,
    pub duration: Duration,
}

impl Cost {
    // Dominates in the Pareto-Sense
    fn dominates(&self, other: &Self) -> bool {
        self.penalty > other.penalty
            && self.duration > other.duration
    }
}

impl Add for Cost {
    type Output = Cost;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            penalty: self.penalty + rhs.penalty,
            duration: self.duration + rhs.duration,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct NodeHeapItem {
    stop_id: u32,
    time: Duration,
    id: NodeIndex,
}

impl Ord for NodeHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stop_id.cmp(&other.stop_id).then(
            self.time.cmp(&other.time)
        )
    }
}

impl PartialOrd for NodeHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.stop_id.partial_cmp(&other.stop_id) {
            Some(ord) => {
                match ord {
                    Ordering::Less | Ordering::Greater => Some(ord),
                    Ordering::Equal => {
                        self.time.partial_cmp(&other.time).map(|ord| ord)
                    }
                }
            }
            None => {
                self.time.partial_cmp(&other.time).map(|ord| ord)
            }
        }
    }
}


pub struct TransitNetwork {
    pub graph: TransitNetworkGraph,
    pub transfer_node_ids: Vec<NodeIndex>,
}

impl TryFrom<DirectConnections> for TransitNetwork {
    type Error = PreprocessingError;

    fn try_from(DirectConnections { lines, .. }: DirectConnections) -> Result<Self, Self::Error> {
        // Build graph
        let (graph, transfer_node_ids) = {
            let mut graph = Graph::new();
            let mut transfer_node_ids: BinaryHeap<NodeHeapItem> = BinaryHeap::new();
            let mut arrival_node_ids: BinaryHeap<NodeHeapItem> = BinaryHeap::new();

            let mut previous_trip_id: Option<u32> = None;
            let mut previous_departure_node_id: Option<NodeIndex> = None;
            let mut previous_departure_time: Option<Duration> = None;

            lines.clone().lazy() // TODO: Remove clone
                .select(vec![
                    col("stop_id"), col("trip_id"), col("line_id"),
                    col("arrival_time"), col("departure_time"), col("stop_sequence"),
                ])
                .sort(
                    ["trip_id", "stop_sequence"],
                    SortMultipleOptions::default()
                        .with_order_descending(false)
                        .with_nulls_last(true)
                )
                .collect()?.iter_chunks(CompatLevel::newest(), false)
                .for_each(|chunk| {
                    let stop_ids: &UInt32Array = chunk.arrays()[0].as_any().downcast_ref().unwrap();
                    let trip_ids: &UInt32Array = chunk.arrays()[1].as_any().downcast_ref().unwrap();
                    let line_ids: &UInt32Array = chunk.arrays()[2].as_any().downcast_ref().unwrap();
                    let arrival_times: &Int64Array = chunk.arrays()[3].as_any().downcast_ref().unwrap();
                    let departure_times: &Int64Array = chunk.arrays()[4].as_any().downcast_ref().unwrap();

                    for i in 0..stop_ids.len() {
                        let stop_id = stop_ids.value(i);
                        let trip_id = trip_ids.value(i);
                        let line_id = line_ids.value(i);
                        let arrival_time = Duration::from_millis(arrival_times.value(i) as u64);
                        let departure_time = Duration::from_millis(departure_times.value(i) as u64);

                        let arrival_node = graph.add_node(Arrival {
                            arrival_time,
                            stop_id,
                        });
                        arrival_node_ids.push(NodeHeapItem {
                            time: arrival_time,
                            stop_id,
                            id: arrival_node,
                        });

                        let departure_node = graph.add_node(Departure {
                            departure_time,
                            stop_id,
                        });
                        let transfer_node = graph.add_node(Transfer {
                            time: departure_time,
                            stop_id,
                        });
                        transfer_node_ids.push(NodeHeapItem {
                            time: departure_time,
                            stop_id,
                            id: transfer_node,
                        });

                        // Insert stay on edge
                        graph.add_edge(arrival_node, departure_node, StayOn { duration: departure_time - arrival_time });
                        // Insert boarding edge
                        graph.add_edge(transfer_node, departure_node, Board);

                        // Insert riding edge
                        // Check if we're still on the same trip and if yes, insert an edge representing riding from the previous station
                        if previous_trip_id == Some(trip_id) {
                            graph.add_edge(
                                previous_departure_node_id.expect("Previous station node id can't be None, since previous trip is Some"),
                                arrival_node,
                                Ride { line_id, duration: arrival_time - previous_departure_time.unwrap() },
                            );
                        }

                        previous_trip_id = Some(trip_id);
                        previous_departure_node_id = Some(departure_node);
                        previous_departure_time = Some(departure_time);
                    }
                });

            let transfer_node_ids = transfer_node_ids.into_sorted_vec();

            // Insert waiting chain (connections between transfer nodes of a station)
            let mut previous_node: Option<NodeHeapItem> = None;

            transfer_node_ids.iter().for_each(|current_node| {
                if let Some(previous_node) = previous_node {
                    if previous_node.stop_id == current_node.stop_id {
                        graph.add_edge(
                            previous_node.id,
                            current_node.id,
                            Waiting { duration: current_node.time - previous_node.time },
                        );
                    }
                }

                previous_node = Some(*current_node);
            });

            // Insert arrival -> transfer ("alight") edges
            let mut start_offset = 0; // Will allow to skip transfer nodes that have too small station id
            arrival_node_ids.into_sorted_vec().into_iter()
                .for_each(|arrival_node| {
                    // Look for the corresponding transfer node (the one that's earliest after arrival on that stop_id)
                    // Since transfer nodes are also already sorted ascending, always take the first one after arrival time
                    let mut earliest_transfer_node = None;
                    for (offset, transfer_node) in transfer_node_ids.iter().skip(start_offset).enumerate() {
                        if transfer_node.stop_id > arrival_node.stop_id {
                            start_offset += offset; // Reduce the search space, since all previous stop_ids are smaller
                            break;
                        }
                        if transfer_node.time >= arrival_node.time + Duration::from_secs(MIN_TRANSFER_TIME_SECS) {
                            earliest_transfer_node = Some(transfer_node);
                            start_offset += offset; // Reduce the search space, since all previous times are smaller
                            break;
                        }
                    }

                    // Add the edge from arrival node to transfer node
                    if let Some(transfer_node) = earliest_transfer_node {
                        graph.add_edge(arrival_node.id, transfer_node.id, Alight { duration: transfer_node.time - arrival_node.time });
                    }
                });

            // TODO: Insert walking edges

            let transfer_node_ids = transfer_node_ids.into_iter()
                .map(|n| n.id)
                .collect();

            (graph, transfer_node_ids)
        };

        Ok(Self {
            graph,
            transfer_node_ids,
        })
    }
}