/// ## Total Supply of Native Token Metric. token_total_supply
/// This metric tracks the total supply of the native token on the Namada blockchain.
/// * token_total_supply: A monotonic counter that records the latest total supply of the native token.
///
/// ### Example
/// ```
/// # HELP token_total_supply The latest total supply of the native token recorded  
/// # TYPE token_total_supply counter  
/// token_total_supply 1000000000  
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

use super::MetricTrait;

pub struct TokenTotalSupply {
    /// The total supply native token
    pub token_total_supply: GaugeVec,
}

impl MetricTrait for TokenTotalSupply {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.token_total_supply.clone()))?;
        Ok(())
    }

    fn update(&self, state: &State) {
        let last_state = state.last_block();

        for supply in &last_state.supplies {
            self.token_total_supply
                .with_label_values(&[&supply.token, &last_state.block.epoch.to_string()])
                .set(supply.total as f64);
        }
    }
}

impl Default for TokenTotalSupply {
    fn default() -> Self {
        let token_total_supply_opts =
            Opts::new("token_total_supply", "The token total supply per epoch");
        Self {
            token_total_supply: GaugeVec::new(token_total_supply_opts, &["token", "epoch"])
                .expect("unable to create token_total_supply metric"),
        }
    }
}
