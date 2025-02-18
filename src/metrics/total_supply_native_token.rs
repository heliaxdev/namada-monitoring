/// ## Total Supply of Native Token Metric. total_supply_native_token
/// This metric tracks the total supply of the native token on the Namada blockchain.
/// * total_supply_native_token: A monotonic counter that records the latest total supply of the native token.
///
/// ### Example
/// ```
/// # HELP total_supply_native_token The latest total supply of the native token recorded  
/// # TYPE total_supply_native_token counter  
/// total_supply_native_token 1000000000  
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::core::{AtomicU64, GenericCounter};
use prometheus_exporter::prometheus::Registry;

use super::MetricTrait;

pub struct TotalSupplyNativeToken {
    /// The latest total supply native token recorded
    pub total_supply_native_token: GenericCounter<AtomicU64>,
}

impl MetricTrait for TotalSupplyNativeToken {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.total_supply_native_token.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.total_supply_native_token.reset();
        self.total_supply_native_token
            .inc_by(state.get_total_supply_native_token());
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        self.total_supply_native_token.inc_by(
            post_state.get_total_supply_native_token() - pre_state.get_total_supply_native_token(),
        );
    }
}

impl Default for TotalSupplyNativeToken {
    fn default() -> Self {
        Self {
            total_supply_native_token: GenericCounter::<AtomicU64>::new(
                "total_supply_native_token",
                "the latest total supply native token recorded",
            )
            .expect("unable to create counter total supply"),
        }
    }
}
