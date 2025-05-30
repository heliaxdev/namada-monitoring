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
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

use super::MetricTrait;

pub struct Signatures {
    /// GaugeVec to track the number of signatures in the lastest block height
    signatures: GaugeVec,
}

impl MetricTrait for Signatures {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.signatures.clone()))?;
        Ok(())
    }

    fn update(&self, state: &State) {
        let last_state = state.last_block();

        let total_signatures = last_state
            .block
            .block
            .last_commit()
            .clone()
            .unwrap()
            .signatures
            .len();
        self.signatures
            .with_label_values(&[&last_state.block.height.to_string()])
            .set(total_signatures as f64);
    }
}

impl Default for Signatures {
    fn default() -> Self {
        let signature_opts = Opts::new(
            "block_signatures",
            "Number of validators signatures per block",
        );
        Self {
            signatures: GaugeVec::new(signature_opts, &["height"])
                .expect("unable to create signatures metric"),
        }
    }
}
