/// ## Transfer Amount (transfer_amount)
/// This metric tracks the total amount of tokens transferred since the monitoring started.
/// It helps monitor token movement trends and detect unusual transfer activity.
///
/// ### Example
/// ```
/// # HELP transfer_amount Token transfer amount
/// # TYPE transfer_amount gauge
/// transfer_amount{token=“NAM”} 5000
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

use super::MetricTrait;

pub struct Transfers {
    /// Transfer amount by token and epoch
    pub transfer_amount: GaugeVec,
}

impl MetricTrait for Transfers {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.transfer_amount.clone()))?;
        Ok(())
    }

    fn update(&self, state: &State) {
        let last_state = state.last_block();

        for transfer in state.get_all_transfers() {
            self.transfer_amount
                .with_label_values(&[&transfer.token, &last_state.block.epoch.to_string()])
                .add(transfer.amount as f64);
        }
    }
}

impl Default for Transfers {
    fn default() -> Self {
        let transfer_amount_opts = Opts::new("transfer_amount", "Token transfer amount");
        let transfer_amount = GaugeVec::new(transfer_amount_opts, &["token", "epoch"])
            .expect("unable to create transaction transfer amount");
        Self { transfer_amount }
    }
}
