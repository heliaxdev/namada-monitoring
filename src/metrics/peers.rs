use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

use super::MetricTrait;

pub struct Peers {
    /// Count how many peers are known
    pub peers: GaugeVec,
}

impl MetricTrait for Peers {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.peers.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        for peer in state.get_all_peers() {
            let version = peer.node_info.moniker.as_ref();
            self.peers.with_label_values(&[version]).add(1_f64);
        }
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
    }
}

impl Default for Peers {
    fn default() -> Self {
        let peers_opts = Opts::new("peers", "Number of peers known");
        let peers =
            GaugeVec::new(peers_opts, &["moniker"]).expect("unable to create gauge vec for peers");
        Self { peers }
    }
}
