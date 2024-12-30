pub mod block;
pub mod epoch;

use block::BlockHeigtCheck;
use epoch::EpochCheck;


pub enum Checks {
    BlockHeightCheck(BlockHeigtCheck),
    EpochCheck(EpochCheck),
}

pub fn all_checks() -> Vec<Checks> {
    vec![
        Checks::BlockHeightCheck(BlockHeigtCheck::default()),
        Checks::EpochCheck(EpochCheck::default()),
    ]
}
