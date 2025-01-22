use crate::state::State;
use anyhow::anyhow;

#[derive(Clone, Debug, Default)]
pub struct EpochCheck {}

impl EpochCheck {
    pub async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        let pre_epoch = pre_state.get_last_block().epoch;
        let post_epoch = post_state.get_last_block().epoch;
        if pre_epoch == post_epoch || pre_epoch.checked_add(1).unwrap_or_default() == post_epoch {
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
