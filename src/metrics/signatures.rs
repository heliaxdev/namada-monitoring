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
use prometheus_exporter::prometheus::{GaugeVec, Opts};

pub struct Signatures {
    /// Histogram of the number of validators that signed the block
    signatures_histogram: Histogram,
    /// GaugeVec to track the number of signatures with the block height as a label
    signatures_gauge: GaugeVec,
}

impl MetricTrait for Signatures {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.signatures_histogram.clone()))?;
        registry.register(Box::new(self.signatures_gauge.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
        // Histograms do not have a reset method, so we do nothing here
        self.signatures_gauge.reset();
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        let delta = post_state.get_block().block.evidence.iter().count() as u64;
        self.signatures_histogram.observe(delta as f64);

        let height = post_state.get_block().block.header.height;
        self.signatures_gauge.with_label_values(&[&height.to_string()]).set(delta as f64);
    }
}

impl Default for Signatures {
    fn default() -> Self {
        let signatures_opts = HistogramOpts::new("signatures", "Number of validators that signed the block");
        let signatures_histogram = Histogram::with_opts(signatures_opts).expect("unable to create histogram for signatures");

        let signatures_gauge_opts = Opts::new("signatures_count", "Number of validators signatures per block");
        let signatures_gauge = GaugeVec::new(signatures_gauge_opts, &["height"]).expect("unable to create gauge for signatures count");

        Self {
            signatures_histogram,
            signatures_gauge,
        }
    }
}
