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
