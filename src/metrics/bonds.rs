/// ## Bonds and Unbonds per epoch metrics. (bonds_per_epoch, unbonds_per_epoch)
/// These metrics track the number of bonds and unbonds per epoch in the Namada blockchains. They help monitor staking activity, validator participation, and network security dynamics. These metrics are gauges, updated at the start of each epoch based on the blockchain state.
/// * bonds_per_epoch: Measures the total amount of tokens bonded in a given epoch.
/// * unbonds_per_epoch: Measures the total amount of tokens unbonded in a given epoch.
///
/// ### Example
/// ```
/// # HELP namada_bonds_per_epoch Total bonds per epoch
/// # TYPE namada_bonds_per_epoch gauge
/// namada_bonds_per_epoch{epoch="2160",chain_id="local.300e84e1e16080e34547d538"} 120000000000
/// # HELP namada_unbonds_per_epoch Total unbonds per epoch
/// # TYPE namada_unbonds_per_epoch gauge
/// namada_unbonds_per_epoch{epoch="2160",chain_id="local.300e84e1e16080e34547d538"} 0
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Gauge, Registry};

use super::MetricTrait;

pub struct Bonds {
    /// Bonds
    bonds: Gauge,
    /// Unbonds
    unbonds: Gauge,
}

impl MetricTrait for Bonds {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.bonds.clone()))?;
        registry.register(Box::new(self.unbonds.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.bonds.set(state.get_future_bonds() as f64);
        self.unbonds.set(state.get_future_unbonds() as f64);
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
    }
}

impl Default for Bonds {
    fn default() -> Self {
        Self {
            bonds: Gauge::new("bonds", "Total bonds in last epoch").expect("unable to create bond"),
            unbonds: Gauge::new("unbonds", "Total unbonds in last epoch")
                .expect("unable to create unbond"),
        }
    }
}
