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
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

use super::MetricTrait;

pub struct Slashes {
    slashes: GaugeVec,
}

impl MetricTrait for Slashes {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.slashes.clone()))?;
        Ok(())
    }

    fn update(&self, state: &State) {
        let last_state = state.last_block();

        let total_slashes = last_state.block.block.evidence.iter().len();
        self.slashes
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(total_slashes as f64);
    }
}

impl Default for Slashes {
    fn default() -> Self {
        let slashes_opts = Opts::new("slashes", "Number of slashes per epoch");
        Self {
            slashes: GaugeVec::new(slashes_opts, &["epoch"])
                .expect("unable to create slashes metric"),
        }
    }
}
