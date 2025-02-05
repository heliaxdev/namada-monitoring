use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Histogram, HistogramOpts, Registry};

use super::MetricTrait;

pub struct BlockTime {
    /// The time spent processing block 
    block_time: Histogram,
}

impl BlockTime {
    pub fn default() -> Self {
        let block_time_opts = HistogramOpts::new(
            "block_time",
            "The time spent processing block",
        )
        .buckets(vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0]);
        let block_time = Histogram::with_opts(block_time_opts)
            .expect("unable to create histogram blocks used time");

        Self {
            block_time
        }
    }
}

impl MetricTrait for BlockTime {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.block_time.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        self.block_time.observe((post_state.get_block().timestamp - pre_state.get_block().timestamp) as f64);
    }
}
