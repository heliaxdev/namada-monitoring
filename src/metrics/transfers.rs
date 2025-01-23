use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

pub struct Transfers {
    /// Transfer amount by token and epoch
    pub transfer_amount: GaugeVec,
}

impl Transfers {
    pub fn default() -> Self {
        let transfer_amount_opts = Opts::new("transfer_amount", "Token transfer amount");
        let transfer_amount = GaugeVec::new(transfer_amount_opts, &["token", "epoch"])
            .expect("unable to create histogram transaction sizes");
        Self { transfer_amount }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.transfer_amount.clone()))?;
        Ok(())
    }

    pub fn reset(&self, state: &State) {
        // FIXME: may not be at an epoch boundary when it starts
        // TODO check if successful tx wrapper and inner
        let transfers = state.get_all_transfers();
        for transfer in transfers {
            self.transfer_amount
                .with_label_values(&[&transfer.token, &state.get_epoch().to_string()])
                .add(transfer.amount as f64);
        }
    }

    pub fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
    }
}
