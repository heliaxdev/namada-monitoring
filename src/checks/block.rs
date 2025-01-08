use crate::state::State;
use anyhow::{anyhow, Ok};

#[derive(Clone, Debug, Default)]
pub struct BlockHeigtCheck {}

impl BlockHeigtCheck {
    pub async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        let pre_block = pre_state.get_last_block();
        let post_block = post_state.get_last_block();
        if pre_block.height + 1 != post_block.height {
            Err(anyhow!(
                "Blocks are not consecutive: pre {} -> post {}",
                pre_block.height,
                post_block.height
            ))
        } else {
            Ok(())
        }
    }
}

pub struct BlockTimeCheck {
    max_time: u64,
}
impl BlockTimeCheck {
    pub async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        let pre_block = pre_state.get_last_block();
        let post_block = post_state.get_last_block();
        let max_estimated_block_time = pre_state.get_max_block_time_estimate() as u64;
        if max_estimated_block_time > self.max_time {
            Err(anyhow!(
                "Max estimated block time is too high: {}",
                max_estimated_block_time
            ))
        } else {
            let estimated_post_time = pre_state.max_next_block_timestamp_estimate();
            if estimated_post_time < post_block.timestamp {
                Err(anyhow!(
                    "Block at height {} took too long to be produced {} (vs {:?})",
                    post_block.height,
                    post_block.timestamp - pre_block.timestamp,
                    pre_state.get_max_block_time_estimate()
                ))
            } else {
                Ok(())
            }
        }
    }
}
