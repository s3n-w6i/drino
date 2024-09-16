use dashmap::{DashMap, DashSet};
use itertools::Itertools;
use pathfinding::prelude::{build_path, dijkstra_all};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::tp::transit_network::TransitNetwork;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

type TransferPattern = Vec<u32>;

// <(from_stop, to_stop), DashSet<Vec<trips>>>
pub type TransferPatternMap = DashMap<(u32, u32), DashSet<TransferPattern>>;

pub struct TransferPatterns(pub TransferPatternMap);

impl From<TransitNetwork> for TransferPatterns {
    fn from(TransitNetwork { graph, transfer_node_ids, .. }: TransitNetwork) -> Self {
        let mut transfer_patterns: TransferPatternMap = DashMap::new();

        transfer_node_ids.par_iter()
            .for_each(|n| {
                // TODO: Use a-star with geographic heuristic
                let parents = dijkstra_all(n, |n| {
                    graph.neighbors(n.clone())
                        .map(|successor| {
                            let edge_id = graph.find_edge(n.clone(), successor.clone()).unwrap();
                            let weight = graph.edge_weight(edge_id).unwrap().cost().duration;
                            (successor, weight.as_secs())
                        })
                        .collect_vec()
                });

                // All shortest paths from n to any other transfer node
                let paths = transfer_node_ids.par_iter()
                    .map(|target| build_path(target, &parents));

                paths
                    // turn paths into their line_ids
                    .map(|path| {
                        if path.len() > 1 {
                            // n isn't contained in the paths, so in theory we would need to calculate the
                            // first leg "manually". However, the first edge will always be a board/wait
                            // edge, which will not be relevant for any transfer pattern.

                            /*let transfer_pattern = (0..path.len() - 1).into_iter()
                                .map(|i| {
                                    let j = i + 1;
                                    let edge_id = graph.find_edge(path[i], path[j]).unwrap();
                                    let edge = graph.edge_weight(edge_id);
                                    if let Some(edge) = edge {
                                        match edge {
                                            Edge::Ride { line_id, .. } => Some(*line_id),
                                            _ => None
                                        }
                                    } else { None }
                                })
                                // remove None values (from edges that aren't rides)
                                .flatten()
                                // Only keep first ride with id (staying on a vehicle for a lot of stations)
                                .dedup()
                                .collect();
                            (path.last().unwrap().clone(), transfer_pattern)*/

                            (path.last().unwrap().clone(), vec![])
                        } else {
                            (path.last().unwrap().clone(), vec![])
                        }
                    })
                    // Remove too long transfer patterns. In the paper, this is called "3-leg heuristic"
                    // for now, we use 4 instead of 3 for less suboptimal patterns
                    // TODO: This heuristic should be embedded in the dijkstra directly
                    .filter(|(_, pattern)| { pattern.len() <= 4 })
                    // Insert into transfer pattern map
                    .for_each(|(to_node_id, transfer_pattern)| {
                        let from_stop_id = graph.node_weight(n.clone()).unwrap().stop_id();
                        let to_stop_id = graph.node_weight(to_node_id.clone()).unwrap().stop_id();
                        let key = &(*from_stop_id, *to_stop_id);

                        match transfer_patterns.get_mut(key) {
                            None => {
                                transfer_patterns.insert(*key, DashSet::from_iter(vec![transfer_pattern]));
                            }
                            Some(existing_patterns) => {
                                existing_patterns.insert(transfer_pattern);
                            }
                        };
                    });
            });

        Self(transfer_patterns)
    }
}