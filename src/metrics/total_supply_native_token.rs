use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use prometheus_exporter::prometheus::Registry;

pub struct TotalSupplyNativeToken {
    /// The latest total supply native token recorded
    pub total_supply_native_token: GenericCounter<AtomicU64>,
}

impl TotalSupplyNativeToken {
    pub fn default() -> Self {
        Self {
            total_supply_native_token: GenericCounter::<AtomicU64>::new(
                "total_supply_native_token",
                "the latest total supply native token recorded",
            )
            .expect("unable to create counter total supply"),
        }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.total_supply_native_token.clone()))?;
        Ok(())
    }

    pub fn reset(&self, state: &State) {
        self.total_supply_native_token.reset();
        self.total_supply_native_token
            .inc_by(state.get_total_supply_native_token());
    }

    pub fn update(&self, pre_state: &State, post_state: &State) {
        self.total_supply_native_token.inc_by(
            post_state.get_total_supply_native_token() - pre_state.get_total_supply_native_token(),
        );
    }
}
