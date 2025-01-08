use crate::{shared::namada::Address, state::State};
use anyhow::anyhow;

#[derive(Clone, Debug, Default)]
pub struct TotalSupplyNativeCheck {
    token: Address,
}

impl TotalSupplyNativeCheck {
    pub fn new(token: Address) -> Self {
        Self { token }
    }

    pub async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        let pre_total_supply = pre_state.get_total_supply(&self.token).unwrap_or_default();
        let post_total_supply = post_state.get_total_supply(&self.token).unwrap_or_default();

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
