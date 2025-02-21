/// ## Voting Power Metrics (one_third_threshold, two_third_threshold)
/// These metrics track the number of validators required to reach 1/3 and 2/3 of the total voting power. They provide insight into consensus formation and validator distribution in the Namada blockchain.
///
///  - one_third_threshold: The number of validators needed to reach 1/3 of the voting power.
///  - two_third_threshold: The number of validators needed to reach 2/3 of the voting power.
/// ### Example
/// ```
/// # HELP one_third_threshold Number of validators to reach 1/3 of the voting power
/// # TYPE one_third_threshold gauge
/// one_third_threshold 5

/// # HELP two_third_threshold Number of validators to reach 2/3 of the voting power
/// # TYPE two_third_threshold gauge
/// two_third_threshold 12
/// ```
use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Gauge, Registry};

use super::MetricTrait;

pub struct VotingPower {
    pub one_third_threshold: Gauge,
    pub two_third_threshold: Gauge,
}

impl MetricTrait for VotingPower {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.one_third_threshold.clone()))?;
        registry.register(Box::new(self.two_third_threshold.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        self.one_third_threshold.set(
            state
                .validators_with_voting_power(1.0 / 3.0)
                .unwrap_or_default() as f64,
        );
        self.two_third_threshold.set(
            state
                .validators_with_voting_power(2.0 / 3.0)
                .unwrap_or_default() as f64,
        );
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
    }
}

impl Default for VotingPower {
    fn default() -> Self {
        let one_third_threshold = Gauge::new(
            "one_third_threshold",
            "The number of validators to reach 1/3 of the voting power",
        )
        .expect("unable to create counter two third threshold");

        let two_third_threshold = Gauge::new(
            "two_third_threshold",
            "The number of validators to reach 2/3 of the voting power",
        )
        .expect("unable to create counter two third threshold");

        Self {
            one_third_threshold,
            two_third_threshold,
        }
    }
}
