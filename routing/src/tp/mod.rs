use crate::algorithm::RoutingAlgorithm;
use crate::direct_connections::DirectConnections;
use crate::tp::transfer_patterns::TransferPatternsTable;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

mod transfer_patterns;
mod init;

pub struct TransferPatternsAlgorithm {
    direct_connections: DirectConnections,
    pub transfer_patterns: TransferPatternsTable,
}

impl RoutingAlgorithm for TransferPatternsAlgorithm {}
