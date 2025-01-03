pub mod block;
pub mod epoch;
pub mod total_supply_native;
pub mod tx_size;

use block::BlockHeigtCheck;
use epoch::EpochCheck;
use total_supply_native::TotalSupplyNativeCheck;
use tx_size::TxSizeCheck;

pub enum Checks {
    BlockHeightCheck(BlockHeigtCheck),
    EpochCheck(EpochCheck),
    TotalSupplyNative(TotalSupplyNativeCheck),
    TxSize(TxSizeCheck),
}

pub fn all_checks() -> Vec<Checks> {
    vec![
        Checks::BlockHeightCheck(BlockHeigtCheck::default()),
        Checks::EpochCheck(EpochCheck::default()),
        Checks::TotalSupplyNative(TotalSupplyNativeCheck::default()),
        Checks::TxSize(TxSizeCheck::default()),
    ]
}
