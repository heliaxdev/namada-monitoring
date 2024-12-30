pub mod block;
pub mod epoch;



use block::BlockHeigtCheck;
use epoch::EpochCheck;

use crate::state::State;

pub enum Checks {
    BlockHeightCheck(BlockHeigtCheck),
    EpochCheck(EpochCheck),
}

pub trait Check {
    async fn run(&self, pre_state: &State, post_state: &State) -> anyhow::Result<()>;
}

pub fn all_checks() -> Vec<Checks> {
    vec![
        Checks::BlockHeightCheck(BlockHeigtCheck::default()),
        Checks::EpochCheck(EpochCheck::default())
    ]
}