use super::MetricTrait;
use crate::state::State;
use anyhow::Result;
use namada_sdk::proof_of_stake::types::ValidatorState as EnumValidatorState;
use prometheus_exporter::prometheus::{GaugeVec, Opts, Registry};

pub struct ValidatorState {
    pub consensus_validators: GaugeVec,
    pub jailed_validators: GaugeVec,
    pub inactive_validators: GaugeVec,
    pub below_threshold_validators: GaugeVec,
    pub below_capacity_validators: GaugeVec,
}

impl MetricTrait for ValidatorState {
    fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.consensus_validators.clone()))?;
        registry.register(Box::new(self.jailed_validators.clone()))?;
        registry.register(Box::new(self.inactive_validators.clone()))?;
        registry.register(Box::new(self.below_threshold_validators.clone()))?;
        registry.register(Box::new(self.below_capacity_validators.clone()))?;
        Ok(())
    }

    fn update(&self, state: &State) {
        let last_state = state.last_block();

        let mut total_consensus_validators = 0;
        let mut total_jailed_validators = 0;
        let mut total_inactive_validators = 0;
        let mut total_below_threshold_validators = 0;

        for validator in &last_state.validators {
            match validator.state {
                EnumValidatorState::Consensus => {
                    total_consensus_validators += 1;
                }
                EnumValidatorState::Jailed => {
                    total_jailed_validators += 1;
                }
                EnumValidatorState::Inactive => {
                    total_inactive_validators += 1;
                }
                EnumValidatorState::BelowThreshold => {
                    total_below_threshold_validators += 1;
                }
                EnumValidatorState::BelowCapacity => {
                    total_below_threshold_validators += 1;
                }
            }
        }

        self.consensus_validators
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(total_consensus_validators as f64);
        self.jailed_validators
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(total_jailed_validators as f64);
        self.inactive_validators
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(total_inactive_validators as f64);
        self.below_threshold_validators
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(total_below_threshold_validators as f64);
        self.below_capacity_validators
            .with_label_values(&[&last_state.block.epoch.to_string()])
            .set(total_below_threshold_validators as f64);
    }
}

impl Default for ValidatorState {
    fn default() -> Self {
        let opts = vec![
            (
                "consensus_validators",
                "Number of validators that are in consensus state",
                &["epoch"],
            ),
            (
                "jailed_validators",
                "Number of validators that are in jailed state",
                &["epoch"],
            ),
            (
                "inactive_validators",
                "Number of validators that are in inactive state",
                &["epoch"],
            ),
            (
                "below_threshold_validators",
                "Number of validators that are below the voting power threshold",
                &["epoch"],
            ),
            (
                "below_capacity_validators",
                "Number of validators that are below the capacity threshold",
                &["epoch"],
            ),
        ];

        let mut metrics = vec![];
        for (name, description, labels) in opts {
            let opts = Opts::new(name, description);
            let gauge = GaugeVec::new(opts, labels)
                .unwrap_or_else(|_| panic!("unable to create {} metric", name));
            metrics.push(gauge);
        }

        Self {
            consensus_validators: metrics[0].clone(),
            jailed_validators: metrics[1].clone(),
            inactive_validators: metrics[2].clone(),
            below_threshold_validators: metrics[3].clone(),
            below_capacity_validators: metrics[4].clone(),
        }
    }
}
