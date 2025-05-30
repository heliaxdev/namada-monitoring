use std::time::Duration;

use super::{AppConfig, CheckTrait};

const POS_ONE_THIRD_CHECK_ID: &str = "pos_one_third_check";
const POS_TWO_THIRD_CHECK_ID: &str = "pos_one_third_check";
const POS_BONDS_CHECK_ID: &str = "pos_bonds_check";
const POS_UNBONDS_CHECK_ID: &str = "pos_unbonds_check";
const POS_CONSENSUS_CHECK_ID: &str = "pos_consensus_check";

pub struct PoSCheck {
    mininimum_one_third_validators: u64,
    mininimum_two_third_validators: u64,
    bond_increase_threshold: f64,
    unbond_increase_threshold: f64,
    consensus_threshold: f64,
}

impl PoSCheck {
    pub fn new(config: &AppConfig) -> Self {
        let config = config.get_config();
        Self {
            mininimum_one_third_validators: config.pos.mininimum_one_third_validators,
            mininimum_two_third_validators: config.pos.mininimum_two_third_validators,
            bond_increase_threshold: config.pos.bond_increase_threshold,
            unbond_increase_threshold: config.pos.unbond_increase_threshold,
            consensus_threshold: config.pos.consensus_threshold,
        }
    }
}

#[async_trait::async_trait]
impl CheckTrait for PoSCheck {
    async fn check(&self, state: &super::State) -> Vec<crate::shared::alert::Alert> {
        let last_state = state.last_block();
        let prev_state = state.prev_block();

        let one_third_validators = state
            .validators_with_voting_power(1.0 / 3.0)
            .expect("Should be able to get validators");
        let two_third_validators = state
            .validators_with_voting_power(2.0 / 3.0)
            .expect("Should be able to get validators");

        let mut alerts = Vec::new();
        if one_third_validators < self.mininimum_one_third_validators {
            alerts.push(crate::shared::alert::Alert {
                check_id: POS_ONE_THIRD_CHECK_ID.to_string(),
                title: "Centralized voting power".to_string(),
                description: format!(
                    "Only {} validators are needed to reach 1/3 of the voting power",
                    one_third_validators
                ),
                metadata: crate::shared::alert::Metadata {
                    block_height: Some(last_state.block.height as u32),
                    tx_id: None,
                },
                severity: crate::shared::alert::Severity::Low,
                trigger_after: Some(Duration::from_secs(60 * 60 * 6)),
                continous: self.is_continous(),
            });
        }

        if two_third_validators < self.mininimum_two_third_validators {
            alerts.push(crate::shared::alert::Alert {
                check_id: POS_TWO_THIRD_CHECK_ID.to_string(),
                title: "Centralized voting power".to_string(),
                description: format!(
                    "Only {} validators are needed to reach 2/3 of the voting power",
                    two_third_validators
                ),
                metadata: crate::shared::alert::Metadata {
                    block_height: Some(last_state.block.height as u32),
                    tx_id: None,
                },
                severity: crate::shared::alert::Severity::Low,
                trigger_after: Some(Duration::from_secs(60 * 60 * 6)),
                continous: self.is_continous(),
            });
        }

        let current_consensus_validators = last_state.consensus_validators().len() as f64;
        let prev_consensus_validators = prev_state.consensus_validators().len() as f64;

        if current_consensus_validators < prev_consensus_validators * self.consensus_threshold {
            alerts.push(crate::shared::alert::Alert {
                check_id: POS_CONSENSUS_CHECK_ID.to_string(),
                title: "Low consensus validators".to_string(),
                description: format!(
                    "Consensus validators dropped from {} to {} in epoch {}",
                    prev_consensus_validators, current_consensus_validators, last_state.block.epoch
                ),
                metadata: crate::shared::alert::Metadata {
                    block_height: Some(last_state.block.height as u32),
                    tx_id: None,
                },
                severity: crate::shared::alert::Severity::Medium,
                trigger_after: Some(Duration::from_secs(60 * 60 * 6)),
                continous: self.is_continous(),
            });
        }

        let current_bonds = last_state.bonds as f64;
        let prev_bonds = prev_state.bonds as f64;
        if prev_bonds + prev_bonds * self.bond_increase_threshold < current_bonds {
            alerts.push(crate::shared::alert::Alert {
                check_id: POS_BONDS_CHECK_ID.to_string(),
                title: "High bond increase".to_string(),
                description: format!(
                    "Bonds increased from {} to {} in epoch {}",
                    prev_bonds, current_bonds, last_state.block.epoch
                ),
                metadata: crate::shared::alert::Metadata {
                    block_height: Some(last_state.block.height as u32),
                    tx_id: None,
                },
                severity: crate::shared::alert::Severity::Low,
                trigger_after: Some(Duration::from_secs(60 * 60)),
                continous: self.is_continous(),
            });
        }

        let current_unbonds = last_state.unbonds as f64;
        let prev_unbonds = prev_state.unbonds as f64;
        if prev_unbonds + prev_unbonds * self.unbond_increase_threshold < current_unbonds {
            alerts.push(crate::shared::alert::Alert {
                check_id: POS_UNBONDS_CHECK_ID.to_string(),
                title: "High unbond increase".to_string(),
                description: format!(
                    "Bonds increased from {} to {} in epoch {}",
                    prev_unbonds, current_unbonds, last_state.block.epoch
                ),
                metadata: crate::shared::alert::Metadata {
                    block_height: Some(last_state.block.height as u32),
                    tx_id: None,
                },
                severity: crate::shared::alert::Severity::Low,
                trigger_after: Some(Duration::from_secs(60 * 60)),
                continous: self.is_continous(),
            });
        }

        alerts
    }

    fn is_continous(&self) -> bool {
        false
    }
}
