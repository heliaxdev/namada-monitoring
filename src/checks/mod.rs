pub mod block;
pub mod epoch;
pub mod total_supply_native;

use block::BlockHeigtCheck;
use block::BlockTimeCheck;
use epoch::EpochCheck;
use total_supply_native::TotalSupplyNativeCheck;

pub enum Checks {
    BlockHeightCheck(BlockHeigtCheck),
    BlockTimeCheck(BlockTimeCheck),
    EpochCheck(EpochCheck),
    TotalSupplyNative(TotalSupplyNativeCheck),
}

pub fn all_checks() -> Vec<Checks> {
    vec![
        Checks::BlockHeightCheck(BlockHeigtCheck::default()),
        Checks::EpochCheck(EpochCheck::default()),
        Checks::TotalSupplyNative(TotalSupplyNativeCheck::default()),
    ]
}
