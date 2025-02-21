/// ## Peers Metric (peers)
/// This metric tracks the number of active peers known to the node. It helps monitor network connectivity, ensuring the node is maintaining
/// a healthy number of connections to other peers. Eg. a bunch of nodes going offline at once - could be a sign of some sort of DoS attack
///
/// * The metric is a gauge, meaning it can increase or decrease based on peer availability.
/// * Each peer is labeled with its moniker

/// ### Example
/// ```
/// # HELP namada_peers Number of peers known
/// # TYPE namada_peers gauge
/// namada_peers{moniker="technodrome-v1.0.0",chain_id="namada.5f5de2dd1b88cba30586420"} 73
/// namada_peers{moniker="technodrome-v1.0.0-dirty",chain_id="namada.5f5de2dd1b88cba30586420"} 3
/// namada_peers{moniker="technodrome-v1.0.1-sec.2",chain_id="namada.5f5de2dd1b88cba30586420"} 21
/// ```
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
