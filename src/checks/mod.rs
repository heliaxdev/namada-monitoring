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

use std::fmt::Display;

use async_trait::async_trait;

pub use crate::config::AppConfig;
pub use crate::state::State;
use crate::{
    checks::{
        block::BlockCheck, fees::FeeCheck, gas::GasCheck, halt::HaltCheck, ibc::IbcCheck, ibc_limit::IbcLimitCheck, pos::PoSCheck, slashes::SlashCheck, transfer_limit::TransferLimitCheck, tx::TxCheck
    },
    shared::alert::Alert,
};

#[async_trait]
pub trait CheckTrait: Send + Sync + Display {
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

    pub fn get_checks(&self) -> &Vec<Box<dyn CheckTrait>> {
        &self.checks
    }
}

impl Display for CheckManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for check in &self.checks {
            write!(f, "{}", check)?;
        }
        write!(f, "CheckManager with {} checks", self.checks.len())
    }
}