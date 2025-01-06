use anyhow::anyhow;

#[derive(Clone, Debug, Default)]
pub struct BlockHeigtCheck {}

impl BlockHeigtCheck {
    pub async fn run(
        &self,
        pre_state: &crate::state::State,
        post_state: &crate::state::State,
    ) -> anyhow::Result<()> {
        match (
            pre_state.latest_block_height,
            post_state.latest_block_height,
        ) {
            (None, None) => Ok(()),    // nothing happen yet
            (None, Some(_)) => Ok(()), // first block
            (Some(pre_block_height), None) => Err(anyhow!(
                "Invalid state: pre {} -> post None",
                pre_block_height
            )), // should never happen
            (Some(pre_block_height), Some(post_block_height)) => {
                if pre_block_height < post_block_height {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Invalid block height: pre {} -> post {}",
                        pre_block_height,
                        post_block_height
                    ))
                }
            }
        }
    }
}


pub struct BlockTimeCheck{ max_time: u64 }
impl BlockTimeCheck {
    pub async fn run(
        &self,
        pre_state: &mut crate::state::State,
        post_state: &mut crate::state::State,
    ) -> anyhow::Result<()> {
        let pre_block = pre_state.get_last_block().cloned();
        let post_block = post_state.get_last_block().cloned();
        match (
            pre_block,
            post_block,
        ) {
            (Some(pre_block), Some(post_block)) => {
                if pre_block.height + 1 != post_block.height {
                    Err(anyhow!("Blocks are not consecutive: pre {} -> post {}", pre_block.height, post_block.height))
                } else {
                    let max_estimated_block_time = pre_state.latest_max_block_time_estimate.unwrap_or(self.max_time);
                    if max_estimated_block_time > self.max_time {
                        Err(anyhow!("Max estimated block time is too high: {}", max_estimated_block_time))
                    } else {
                        let max_estimated_block_time = max_estimated_block_time as i64;
                        let estimated_post_time = pre_block.timestamp + max_estimated_block_time;
                        if estimated_post_time < post_block.timestamp {
                            Err(anyhow!("Block at height {} took too long to be produced {} (vs {:?})", post_block.height, post_block.timestamp - pre_block.timestamp, pre_state.latest_max_block_time_estimate))
                        } else {
                            Ok(())
                        }
                    }
                }
            },
            _ => Ok(()),

        }
    }
}