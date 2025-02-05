use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use prometheus_exporter::prometheus::Registry;

use super::MetricTrait;

pub struct EpochCounter {
    /// The latest epoch recorded
    epoch_counter: GenericCounter<AtomicU64>,
}

impl MetricTrait for EpochCounter {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.epoch_counter.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.epoch_counter.reset();
        self.epoch_counter.inc_by(state.get_last_block().epoch);
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        self.epoch_counter
            .inc_by(post_state.get_last_block().epoch - pre_state.get_last_block().epoch);
    }
}

impl EpochCounter {
    pub fn default() -> Self {
        Self {
            epoch_counter: GenericCounter::<AtomicU64>::new("epoch", "the latest epoch recorded")
                .expect("unable to create counter epoch"),
        }
    }
}
