use crate::algorithms::RoutingAlgorithm;
use crate::direct_connections::DirectConnections;
use crate::tp::transfer_pattern_ds::table::TransferPatternsTable;

mod init;
mod querying;
pub(crate) mod transfer_pattern_ds;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

pub struct TransferPatternsAlgorithm {
    pub direct_connections: DirectConnections,
    pub transfer_patterns: TransferPatternsTable,
}

impl RoutingAlgorithm for TransferPatternsAlgorithm {}
