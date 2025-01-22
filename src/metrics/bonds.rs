use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

pub struct Bonds {
    /// Bonds per epoch
    bonds_per_epoch: GaugeVec,
    /// Unbonds per epoch
    unbonds_per_epoch: GaugeVec,
}

impl Bonds {
    pub fn default() -> Self {
        let bonds_per_epoch_opts = Opts::new("bonds_per_epoch", "Total bonds per epoch");

        let unbonds_per_epoch_opts = Opts::new("unbonds_per_epoch", "Total unbonds per epoch");

        Self {
            bonds_per_epoch: GaugeVec::new(bonds_per_epoch_opts, &["epoch"])
                .expect("unable to create gauge vec for bond_per_epoch"),
            unbonds_per_epoch: GaugeVec::new(unbonds_per_epoch_opts, &["epoch"])
                .expect("unable to create gauge vec for unbond_per_epoch"),
        }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.bonds_per_epoch.clone()))?;
        registry.register(Box::new(self.unbonds_per_epoch.clone()))?;
        Ok(())
    }

    pub fn reset(&self, state: &State) {
        self.bonds_per_epoch.reset();
        self.bonds_per_epoch
            .with_label_values(&[&(state.get_epoch() + 1).to_string()])
            .set(state.get_future_bonds() as f64);

        self.unbonds_per_epoch.reset();
        self.unbonds_per_epoch
            .with_label_values(&[&(state.get_epoch() + 1).to_string()])
            .set(state.get_future_unbonds() as f64);
    }

    pub fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
    }
}
