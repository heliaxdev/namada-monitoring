mod block;
mod fees;
mod gas;
mod halt;
mod ibc;
mod ibc_limit;
mod pos;
mod slashes;
mod transfer_limit;
mod tx;

use async_trait::async_trait;
use block::BlockCheck;
use fees::FeeCheck;
use gas::GasCheck;
use halt::HaltCheck;
use ibc::IbcCheck;
use ibc_limit::IbcLimitCheck;
use pos::PoSCheck;
use slashes::SlashCheck;
use transfer_limit::TransferLimitCheck;
use tx::TxCheck;

pub use crate::config::AppConfig;
use crate::shared::alert::Alert;
pub use crate::state::State;

#[async_trait]
pub trait CheckTrait: Send + Sync {
    async fn check(&self, state: &State) -> Vec<Alert>;
    fn is_continous(&self) -> bool;
}

pub struct CheckManager {
    checks: Vec<Box<dyn CheckTrait>>,
}

impl CheckManager {
    pub fn new(config: &AppConfig) -> Self {
        let checks: Vec<Box<dyn CheckTrait>> = vec![
            Box::new(FeeCheck::new(config)),
            Box::new(BlockCheck::new(config)),
            Box::new(PoSCheck::new(config)),
            Box::new(TxCheck::new(config)),
            Box::new(HaltCheck::new(config)),
            Box::new(GasCheck::new(config)),
            Box::new(IbcCheck::new(config)),
            Box::new(TransferLimitCheck::new(config)),
            Box::new(SlashCheck::default()),
            Box::new(IbcLimitCheck::default()),
        ];
        Self { checks }
    }

    pub async fn run_block_checks(&self, states: &State) -> Vec<Alert> {
        let mut results = Vec::new();
        for check in &self.checks {
            if check.is_continous() {
                continue;
            }
            results.extend(check.check(states).await);
        }
        results
    }

    pub async fn run_continous_checks(&self, states: &State) -> Vec<Alert> {
        let mut results = Vec::new();
        for check in &self.checks {
            if check.is_continous() {
                results.extend(check.check(states).await);
            }
        }
        results
    }
}
