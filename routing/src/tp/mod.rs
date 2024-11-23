use crate::algorithm::{PreprocessingResult, RoutingAlgorithm};
use crate::direct_connections::DirectConnections;
use crate::stp::preprocessing::clustering::filter_for_cluster::StopIdMapping;
use crate::tp::transfer_pattern_ds::table::TransferPatternsTable;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

mod init;
mod transfer_pattern_ds;

pub struct TransferPatternsAlgorithm {
    direct_connections: DirectConnections,
    pub transfer_patterns: TransferPatternsTable,
}

impl RoutingAlgorithm for TransferPatternsAlgorithm {}

impl TransferPatternsAlgorithm {
    pub(crate) fn rename_stops(&mut self, mapping: StopIdMapping) -> PreprocessingResult<()> {
        let mapping = mapping.collect()?;
        /*let global_stop_ids = &mapping.column("global_stop_id")?.u32()?.to_vec()
            .into_iter().filter_map(|x| x.map(StopId))
            .collect_vec();
        let stop_ids_in_cluster = &mapping.column("stop_id_in_cluster")?.u32()?.to_vec()
            .into_iter().filter_map(|x| x.map(StopId))
            .collect_vec();*/
        
        self.direct_connections.rename_stops(&mapping)?;
        
        Ok(())
    }
}