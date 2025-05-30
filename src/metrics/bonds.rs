/// ## Bonds and Unbonds per epoch metrics. (bonds, unbonds)
/// These metrics track the number of bonds and unbonds per epoch in the Namada blockchains. They help monitor staking activity, validator participation, and network security dynamics. These metrics are gauges, updated at the start of each epoch based on the blockchain state.
/// * bonds: Measures the total amount of tokens bonded in a given epoch.
/// * unbonds: Measures the total amount of tokens unbonded in a given epoch.
///
/// ### Example
/// ```
/// # HELP namada_bonds Total bonds per epoch
/// # TYPE namada_bonds gauge
/// namada_bonds{epoch="2160",chain_id="local.300e84e1e16080e34547d538"} 120000000000
/// # HELP namada_unbonds Total unbonds per epoch
/// # TYPE namada_unbonds gauge
/// namada_unbonds{epoch="2160",chain_id="local.300e84e1e16080e34547d538"} 0
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

use super::MetricTrait;

pub struct Bonds {
    /// Bonds
    bonds: GaugeVec,
    /// Unbonds
    unbonds: GaugeVec,
}

impl MetricTrait for Bonds {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.bonds.clone()))?;
        registry.register(Box::new(self.unbonds.clone()))?;
        Ok(())
    }

    fn update(&self, state: &State) {
        let last_state = state.last_block();

        self.bonds
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(last_state.bonds as f64);
        self.unbonds
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(last_state.unbonds as f64);
    }
}

impl Default for Bonds {
    fn default() -> Self {
        let bonds_opts = Opts::new("bonds", "Total bonds per epoch");
        let unbonds_opts = Opts::new("unbonds", "Total unbonds per epoch");
        Self {
            bonds: GaugeVec::new(bonds_opts, &["epoch"]).expect("unable to create bonds"),
            unbonds: GaugeVec::new(unbonds_opts, &["epoch"]).expect("unable to create unbond"),
        }
    }
}
