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
use halt::HaltCheck;

pub use crate::config::AppConfig;
pub use crate::state::State;
use crate::{
    checks::{
        block::BlockCheck, fees::FeeCheck, gas::GasCheck, ibc::IbcCheck, ibc_limit::IbcLimitCheck,
        pos::PoSCheck, slashes::SlashCheck, transfer_limit::TransferLimitCheck, tx::TxCheck,
    },
    shared::alert::Alert,
};

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
