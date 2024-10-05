use crate::algorithm::{AllRange, PreprocessInit, RoutingAlgorithm};
use crate::direct_connections::DirectConnections;
use crate::tp::transfer_patterns::{TransferPatternsGraph, TransferPatternsTable};

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

mod transit_network;
mod transfer_patterns;
mod init;

pub struct TransferPatternsAlgorithm {
    direct_connections: DirectConnections,
    pub transfer_patterns: TransferPatternsTable,
}

impl RoutingAlgorithm for TransferPatternsAlgorithm {}
