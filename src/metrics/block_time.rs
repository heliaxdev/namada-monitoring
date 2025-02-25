/// ## Block Processing Time (block_time)
/// This metric tracks the time spent processing a block. It helps monitor the efficiency of block execution
/// and can highlight performance bottlenecks.
///
/// * The metric is a histogram, capturing block processing times in predefined buckets.
/// * Buckets are set at [15, 30, 45, 60, 75, 90, 105, 120, 135, 150, 165, 180, 195, 210, 225, 240] seconds.
///
/// ### Example
///
/// # HELP namada_block_time The time spent processing block
// # TYPE namada_block_time histogram
/// namada_block_time_bucket{le="15"} 0
/// namada_block_time_bucket{le="30"} 0
/// namada_block_time_bucket{le="45"} 0
/// ...
/// namada_block_time_bucket{le="210"} 8
/// namada_block_time_bucket{le="225"} 8
/// namada_block_time_bucket{le="240"} 8
/// 
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Histogram, HistogramOpts, Registry, exponential_buckets};

use super::MetricTrait;

pub struct BlockTime {
    /// The time spent processing block
    block_time: Histogram,
}

impl Default for BlockTime {
    fn default() -> Self {
        // Define exponential buckets: start at 2, grow by factor 1.5, with 10 buckets
        let buckets = exponential_buckets(2.0, 1.5, 15).unwrap();
        let block_time_opts = HistogramOpts::new("block_time", "The time spent processing block")
            .buckets(buckets);
        let block_time = Histogram::with_opts(block_time_opts)
            .expect("unable to create histogram blocks used time");

        Self { block_time }
    }
}

impl MetricTrait for BlockTime {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.block_time.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {}

    fn update(&self, pre_state: &State, post_state: &State) {
        self.block_time
            .observe((post_state.get_block().timestamp - pre_state.get_block().timestamp) as f64);
    }
}
