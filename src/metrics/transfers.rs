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

    fn reset(&self, state: &State) {
        // FIXME: may not be at an epoch boundary when it starts
        let transfers = state.get_all_transfers();
        for transfer in transfers {
            self.transfer_amount
                .with_label_values(&[&transfer.token, &state.get_epoch().to_string()])
                .add(transfer.amount as f64);
        }
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
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
