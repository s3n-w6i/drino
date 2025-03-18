use crate::algorithms::RoutingAlgorithm;

pub(crate) mod preprocessing;

#[derive(Clone)]
pub struct ScalableTransferPatternsAlgorithm;

impl RoutingAlgorithm for ScalableTransferPatternsAlgorithm {}
