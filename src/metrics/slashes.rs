/// ## Slashes Counter (slashes)
/// This metric tracks the number of validators that signed the block, providing visibility into the block
/// signing activity over time.
///
/// ### Example
/// ```text
/// # HELP slashes Number of validators that signed the block
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
use prometheus_exporter::prometheus::{Histogram, HistogramOpts, Registry};

use super::MetricTrait;
use prometheus_exporter::prometheus::{GaugeVec, Opts};

pub struct Slashes {
    /// Histogram of the number of validators that signed the block
    slashes_histogram: Histogram,
    /// GaugeVec to track the number of slashes with the block height as a label
    slashes_gauge: GaugeVec,
}

impl MetricTrait for Slashes {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.slashes_histogram.clone()))?;
        registry.register(Box::new(self.slashes_gauge.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
        // Histograms do not have a reset method, so we do nothing here
        self.slashes_gauge.reset();
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        let total_slashes = post_state.get_slashes();
        self.slashes_histogram.observe(total_slashes as f64);

        let height = post_state.get_block().block.header.height;
        self.slashes_gauge
            .with_label_values(&[&height.to_string()])
            .set(total_slashes as f64);
    }
}

impl Default for Slashes {
    fn default() -> Self {
        let slashes_opts = HistogramOpts::new("slashes", "Number of slashed validators");
        let slashes_histogram =
            Histogram::with_opts(slashes_opts).expect("unable to create histogram for slashes");

        let slashes_gauge_opts =
            Opts::new("slashes_count", "Number of validators slashed per block");
        let slashes_gauge = GaugeVec::new(slashes_gauge_opts, &["height"])
            .expect("unable to create gauge for slashes count");

        Self {
            slashes_histogram,
            slashes_gauge,
        }
    }
}
