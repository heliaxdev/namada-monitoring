use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use prometheus_exporter::prometheus::Registry;

use super::MetricTrait;

pub struct BlockHeightCounter {
    block_height_counter: GenericCounter<AtomicU64>,
}

impl MetricTrait for BlockHeightCounter {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.block_height_counter.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.block_height_counter.reset();
        self.block_height_counter
            .inc_by(state.get_last_block().height);
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        self.block_height_counter
            .inc_by(post_state.get_last_block().height - pre_state.get_last_block().height);
    }
}

impl BlockHeightCounter {
    pub fn default() -> Self {
        Self {
            block_height_counter: GenericCounter::<AtomicU64>::new(
                "block_height",
                "the latest block height recorded",
            )
            .expect("unable to create counter block_height"),
        }
    }
}
