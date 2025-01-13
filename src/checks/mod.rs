pub mod block;
pub mod epoch;
pub mod total_supply_native;
pub mod tx_size;

use crate::config::AppConfig;
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

pub fn all_checks(config: AppConfig) -> Vec<Checks> {
    vec![
        Checks::BlockHeightCheck(BlockHeigtCheck::default()),
        Checks::EpochCheck(EpochCheck::default()),
        Checks::TotalSupplyNative(TotalSupplyNativeCheck::default()),
        Checks::TxSize(TxSizeCheck::new(
            config.max_tx_size,
            config.max_tx_inner_len,
        )),
    ]
}
