/// ## Epoch Counter (epoch)
/// This metric tracks the latest epoch recorded on a Namada blockchain, providing visibility into epoch progression and chain
/// activity over time. A steadily increasing counter indicates normal epoch progression.
///
/// ### Example
/// ```
/// # HELP epoch The latest epoch recorded  
/// # TYPE epoch counter  
/// epoch 256
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use prometheus_exporter::prometheus::Registry;

use super::MetricTrait;

pub struct Epoch {
    /// The latest epoch recorded
    epoch_counter: GenericCounter<AtomicU64>,
}

impl MetricTrait for Epoch {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.epoch_counter.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.epoch_counter.reset();
        self.epoch_counter.inc_by(state.get_last_block().epoch);
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        if pre_state.get_last_block().epoch > post_state.get_last_block().epoch {
            tracing::error!(
                "Epoch not updated Error. pre_state: {:?}, post_state: {:?}",
                pre_state.get_last_block().epoch,
                post_state.get_last_block().epoch
            );
        }
        self.reset(post_state);
    }
}

impl Default for Epoch {
    fn default() -> Self {
        Self {
            epoch_counter: GenericCounter::<AtomicU64>::new("epoch", "the latest epoch recorded")
                .expect("unable to create counter epoch"),
        }
    }
}
