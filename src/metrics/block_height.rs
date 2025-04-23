/// ## Block Height (block_height)
/// This metric tracks the latest block height of a Namada blockchain. It provides a real-time view of
/// block progression, and helps monitor chain liveness and ensure continuous block production.
/// It is updated at each block by fetching the latest block height from the blockchain state.
///
/// ### Example
/// ```
/// # HELP namada_block_height the latest block height recorded
/// # TYPE namada_block_height counter
/// namada_block_height 12960
/// ```
///
/// ## Alert: Block Height Stalled:
/// If no blocks are registered in 5 minutes, the block height has stalled. Alert the team to investigate the issue.
/// ```
/// (time() - min_over_time(time()[5m])) > 300 and (absent_over_time(namada_block_height[5m]) or increase(namada_block_height[5m]) == 0)
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use prometheus_exporter::prometheus::Registry;

use super::MetricTrait;

pub struct BlockHeight {
    block_height: GenericCounter<AtomicU64>,
}

impl MetricTrait for BlockHeight {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.block_height.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.block_height.reset();
        self.block_height.inc_by(state.get_last_block().height);
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        self.block_height
            .inc_by(post_state.get_last_block().height - pre_state.get_last_block().height);
    }
}

impl Default for BlockHeight {
    fn default() -> Self {
        Self {
            block_height: GenericCounter::<AtomicU64>::new(
                "block_height",
                "the latest block height recorded",
            )
            .expect("unable to create counter block_height"),
        }
    }
}
