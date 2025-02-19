/// ## Signature Counter (signatures)
/// This metric tracks the number of validators that signed the block, providing visibility into the block 
/// signing activity over time.
///
/// ### Example
/// ```text
/// # HELP signatures Number of validators that signed the block
/// # TYPE signatures histogram
/// signatures_bucket{le="0.005"} 0
/// signatures_bucket{le="0.01"} 0
/// signatures_bucket{le="0.025"} 0
/// signatures_bucket{le="0.05"} 0
/// signatures_bucket{le="0.1"} 0
/// signatures_bucket{le="0.25"} 0
/// signatures_bucket{le="0.5"} 0
/// signatures_bucket{le="1"} 0
/// signatures_bucket{le="2.5"} 0
/// signatures_bucket{le="5"} 0
/// signatures_bucket{le="10"} 0
/// signatures_bucket{le="+Inf"} 0
/// signatures_sum 0
/// signatures_count 0
/// ```

use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Histogram, HistogramOpts, Registry};

use super::MetricTrait;

pub struct Signatures {
    /// Histogram of the number of validators that signed the block
    signatures_histogram: Histogram,
}

impl MetricTrait for Signatures {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.signatures_histogram.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
        // Histograms do not have a reset method, so we do nothing here
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        let delta = post_state.get_block().block.evidence.iter().count() as u64;
        self.signatures_histogram.observe(delta as f64);
    }
}

impl Default for Signatures {
    fn default() -> Self {
        let signatures_opts = HistogramOpts::new("signatures", "Number of validators that signed the block");
        let signatures_histogram = Histogram::with_opts(signatures_opts).expect("unable to create histogram for signatures");

        Self {
            signatures_histogram,
        }
    }
}
