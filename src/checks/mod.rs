mod fees;
mod slashes;
mod tendermint_rs;

use fees::FeeCheck;
use slashes::SlashCheck;
use tendermint_rs::TendermintRsCheck;

pub use crate::config::AppConfig;
pub use crate::state::State;

pub trait CheckTrait {
    fn check(&self, states: &[&State]) -> Vec<String>;
}

// static list of all defined checks
pub struct CheckExporter {
    checks: Vec<Box<dyn CheckTrait>>,
}

impl CheckExporter {
    pub fn new(config: &AppConfig) -> Self {
        let checks: Vec<Box<dyn CheckTrait>> = vec![
            Box::new(FeeCheck::new(config)),
            Box::new(SlashCheck::default()),
            Box::new(TendermintRsCheck::default()),
        ];
        Self { checks }
    }

    pub fn run_checks(&self, states: &[&State]) -> Vec<String> {
        let mut results = Vec::new();
        for check in &self.checks {
            results.extend(check.check(states));
        }
        results
    }
}
