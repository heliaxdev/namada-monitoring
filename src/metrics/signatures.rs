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
use prometheus_exporter::prometheus::{Gauge, Registry};

use super::MetricTrait;

pub struct Signatures {
    /// GaugeVec to track the number of signatures in the lastest block height
    signatures: Gauge,
}

impl MetricTrait for Signatures {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.signatures.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
        self.signatures.set(0.0);
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        let total_signatures = post_state.get_signatures().len() as u64;
        self.signatures.set(total_signatures as f64);
    }
}

impl Default for Signatures {
    fn default() -> Self {
        let signatures = Gauge::new(
            "signatures",
            "Number of validators signatures per block",
        ).expect("unable to create gauge for signatures count");

        Self {
            // signatures_histogram,
            signatures,
        }
    }
}
