/// ## Block Processing Time (block_time)
/// This metric tracks the time spent processing a block. It helps monitor the efficiency of block execution and can
///  highlight performance bottlenecks.
///
/// * The metric is a histogram, capturing block processing times in predefined buckets.
/// * Buckets are set at [1, 2, 4, 8, 16, 32, 64, 128, 256] milliseconds.
///
/// ### Example
///
/// # HELP namada_block_time The time spent processing block
// # TYPE namada_block_time histogram
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="1"} 0
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="2"} 0
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="4"} 0
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="8"} 5
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="16"} 8
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="32"} 8
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="64"} 8
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="128"} 8
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="256"} 8
/// namada_block_time_bucket{chain_id="housefire-alpaca.cc0d3e0c033be",le="+Inf"} 8
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Histogram, HistogramOpts, Registry};

use super::MetricTrait;

pub struct BlockTime {
    /// The time spent processing block
    block_time: Histogram,
}

impl Default for BlockTime {
    fn default() -> Self {
        let block_time_opts = HistogramOpts::new("block_time", "The time spent processing block")
            .buckets(vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0]);
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
