use anyhow::anyhow;

#[derive(Clone, Debug, Default)]
pub struct TotalSupplyNativeCheck {}

impl TotalSupplyNativeCheck {
    pub async fn run(
        &self,
        pre_state: &crate::state::State,
        post_state: &crate::state::State,
    ) -> anyhow::Result<()> {
        match (
            pre_state.latest_total_supply_native,
            post_state.latest_total_supply_native,
        ) {
            (None, None) => Ok(()),    // nothing happen yet
            (None, Some(_)) => Ok(()), // first epoch
            (Some(pre_total_supply), None) => Err(anyhow!(
                "Invalid state: pre {} -> post None",
                pre_total_supply
            )), // should never happen
            (Some(pre_total_supply), Some(post_total_supply)) => {
                if pre_total_supply <= post_total_supply {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Invalid total supply: pre {} -> post {}. Could be valid in case of slashes or rejected governance proposal.",
                        pre_total_supply,
                        post_total_supply
                    ))
                }
            }
        }
    }
}
