use anyhow::anyhow;

use super::Check;

#[derive(Clone, Debug, Default)]
pub struct EpochCheck {}

impl Check for EpochCheck {
    async fn run(
        &self,
        pre_state: &crate::state::State,
        post_state: &crate::state::State,
    ) -> anyhow::Result<()> {
        match (pre_state.latest_epoch, post_state.latest_epoch) {
            (None, None) => Ok(()),    // nothing happen yet
            (None, Some(_)) => Ok(()), // first epoch
            (Some(pre_epoch), None) => {
                Err(anyhow!("Invalid state: pre {} -> post None", pre_epoch))
            } // should never happen
            (Some(pre_epoch), Some(post_epoch)) => {
                if pre_epoch <= post_epoch {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Invalid epoch: pre {} -> post {}",
                        pre_epoch,
                        post_epoch
                    ))
                }
            }
        }
    }
}
