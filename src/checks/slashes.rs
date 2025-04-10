use namada_sdk::tendermint::evidence;

use super::{CheckTrait, State};

#[derive(Default)]
pub struct SlashCheck {}

impl CheckTrait for SlashCheck {
    fn check(&self, states: &[&State]) -> Vec<String> {
        let last_state = states.last().unwrap();
        let last_block = last_state.get_last_block();
        let mut results = Vec::new();

        for evidence in last_block.block.evidence.iter() {
            match evidence {
                evidence::Evidence::DuplicateVote(duplicate_vote_evidence) => {
                    let duplicate_vote_evidence_description = format!(
                        "Duplicate vote evidence found. Total power: {}. 
                        Validator {} voted {} at height {}.
                        Validator {} voted {} at height {}.",
                        duplicate_vote_evidence.total_voting_power,
                        duplicate_vote_evidence.vote_a.validator_address,
                        duplicate_vote_evidence.vote_a.vote_type,
                        duplicate_vote_evidence.vote_a.height,
                        duplicate_vote_evidence.vote_b.validator_address,
                        duplicate_vote_evidence.vote_b.vote_type,
                        duplicate_vote_evidence.vote_b.height
                    );
                    results.push(duplicate_vote_evidence_description);
                }
                evidence::Evidence::LightClientAttack(light_client_attack_evidence) => {
                    let light_client_attack_evidence_description = format!(
                        "Light client attack evidence found. Total power: {}. 
                        Conflicting block height: {}.
                        Conflicting block proposer: {}. 
                        Common height: {}. 
                        Byzantine validators: {:?}.",
                        light_client_attack_evidence.total_voting_power,
                        light_client_attack_evidence
                            .conflicting_block
                            .signed_header
                            .header
                            .height,
                        light_client_attack_evidence
                            .conflicting_block
                            .signed_header
                            .header
                            .proposer_address,
                        light_client_attack_evidence.common_height,
                        light_client_attack_evidence.byzantine_validators
                    );
                    results.push(light_client_attack_evidence_description);
                }
            }
        }
        results
    }
}
