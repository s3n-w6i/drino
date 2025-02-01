pub(crate) mod preprocessing;

use crate::algorithm::RoutingAlgorithm;

#[derive(Clone)]
pub struct ScalableTransferPatternsAlgorithm;

impl RoutingAlgorithm for ScalableTransferPatternsAlgorithm {}