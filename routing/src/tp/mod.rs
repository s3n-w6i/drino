use crate::algorithm::RoutingAlgorithm;
use crate::direct_connections::DirectConnections;
use crate::tp::transfer_pattern_ds::table::TransferPatternsTable;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

mod init;
pub(crate) mod transfer_pattern_ds;

pub(crate) struct TransferPatternsAlgorithm {
    pub direct_connections: DirectConnections,
    pub transfer_patterns: TransferPatternsTable,
}

impl RoutingAlgorithm for TransferPatternsAlgorithm {}