use super::MetricTrait;
use crate::state::State;
use anyhow::Result;
use namada_sdk::proof_of_stake::types::ValidatorState as EnumValidatorState;
use prometheus_exporter::prometheus::{
    core::{AtomicU64, GenericCounter},
    Registry,
};

pub struct ValidatorState {
    pub consensus_validators: GenericCounter<AtomicU64>,
    pub jailed_validators: GenericCounter<AtomicU64>,
    pub inactive_validators: GenericCounter<AtomicU64>,
    pub below_threshold_validators: GenericCounter<AtomicU64>,
    pub below_capacity_validators: GenericCounter<AtomicU64>,
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

    fn reset(&self, state: &State) {
        self.consensus_validators.reset();
        self.jailed_validators.reset();
        self.inactive_validators.reset();
        self.below_threshold_validators.reset();
        self.below_capacity_validators.reset();

        for validator in state.get_validators() {
            match validator.state {
                EnumValidatorState::Consensus => self.consensus_validators.inc(),
                EnumValidatorState::BelowCapacity => self.below_capacity_validators.inc(),
                EnumValidatorState::BelowThreshold => self.below_threshold_validators.inc(),
                EnumValidatorState::Inactive => self.inactive_validators.inc(),
                EnumValidatorState::Jailed => self.jailed_validators.inc(),
            }
        }
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state);
    }
}

impl Default for ValidatorState {
    fn default() -> Self {
        let consensus_validators = GenericCounter::new(
            "consensus_validators",
            "Number of validators that are in consensus state",
        )
        .expect("unable to create consensus_validators");
        let jailed_validators = GenericCounter::new(
            "jailed_validators",
            "Number of validators that are in jailed state",
        )
        .expect("unable to create jailed_validators");
        let inactive_validators = GenericCounter::new(
            "inactive_validators",
            "Number of validators that are in inactive state",
        )
        .expect("unable to create inactive_validators");
        let below_threshold_validators = GenericCounter::new(
            "below_threshold_validators",
            "Number of validators that are below the voting power threshold",
        )
        .expect("unable to create below_threshold_validators");
        let below_capacity_validators = GenericCounter::new(
            "below_capacity_validators",
            "Number of validators that are below the capacity threshold",
        )
        .expect("unable to create below_capacity_validators");

        Self {
            consensus_validators,
            jailed_validators,
            inactive_validators,
            below_threshold_validators,
            below_capacity_validators,
        }
    }
}
