/// ## Slashes Counter (slashes)
/// This metric tracks the number of validators that has been slashed in the last block.
///
/// ### Example
/// ```text
/// # HELP slashes Number of validators that has been slashed per block
/// # TYPE slashes histogram
/// slashes_bucket{le="0.005"} 0
/// slashes_bucket{le="0.01"} 0
/// slashes_bucket{le="0.025"} 0
/// slashes_bucket{le="0.05"} 0
/// slashes_bucket{le="0.1"} 0
/// slashes_bucket{le="0.25"} 0
/// slashes_bucket{le="0.5"} 0
/// slashes_bucket{le="1"} 0
/// slashes_bucket{le="2.5"} 0
/// slashes_bucket{le="5"} 0
/// slashes_bucket{le="10"} 0
/// slashes_bucket{le="+Inf"} 0
/// slashes_sum 0
/// slashes_count 0
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{
    core::{AtomicU64, GenericCounter},
    Registry,
};

use super::MetricTrait;

pub struct Slashes {
    /// Overall slashes count (will only increase)
    slashes: GenericCounter<AtomicU64>,
}

impl MetricTrait for Slashes {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.slashes.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
        // Histograms do not have a reset method, so we do nothing here
        self.slashes.reset();
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        let total_slashes = post_state.get_slashes();
        self.slashes.inc_by(total_slashes);
    }
}

impl Default for Slashes {
    fn default() -> Self {
        let slashes = GenericCounter::new("slashes", "Number of validators slashed")
            .expect("unable to create counter for slashes count");

        Self { slashes }
    }
}
