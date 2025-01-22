pub mod block;
pub mod epoch;
pub mod total_supply_native;
pub mod tx_size;

use crate::config::AppConfig;
use crate::state::State;
use block::BlockHeigtCheck;
use block::BlockTimeCheck;
use epoch::EpochCheck;
use total_supply_native::TotalSupplyNativeCheck;
use tx_size::TxSizeCheck;

pub enum Checks {
    BlockHeightCheck(BlockHeigtCheck),
    BlockTimeCheck(BlockTimeCheck),
    EpochCheck(EpochCheck),
    TotalSupplyNative(TotalSupplyNativeCheck),
    TxSize(TxSizeCheck),
}

impl Checks {
    pub async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        match self {
            Checks::BlockHeightCheck(check) => check.run(pre_state, post_state).await,
            Checks::BlockTimeCheck(check) => check.run(pre_state, post_state).await,
            Checks::EpochCheck(check) => check.run(pre_state, post_state).await,
            Checks::TotalSupplyNative(check) => check.run(pre_state, post_state).await,
            Checks::TxSize(check) => check.run(pre_state, post_state).await,
        }
    }
}

pub struct CheckCollection {
    checks: Vec<Checks>,
}

impl CheckCollection {
    pub fn new(config: &AppConfig) -> Self {
        let checks = vec![
            Checks::BlockHeightCheck(BlockHeigtCheck::default()),
            Checks::EpochCheck(EpochCheck::default()),
            Checks::TotalSupplyNative(TotalSupplyNativeCheck::default()),
            Checks::TxSize(TxSizeCheck::new(
                config.max_tx_size,
                config.max_tx_inner_len,
            )),
        ];
        Self { checks }
    }

    pub async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()> {
        for check in &self.checks {
            check.run(pre_state, post_state).await?;
        }
        Ok(())
    }
}
