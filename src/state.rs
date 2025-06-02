use std::collections::{BTreeMap, HashMap};

use anyhow::anyhow;
use namada_sdk::{ibc::IbcMessage, proof_of_stake::types::ValidatorState};

use crate::shared::{
    namada::{Block, Inner, InnerKind, Transfer, TransferKind, Validator, Wrapper},
    supply::Supply,
};

#[derive(Debug, Clone)]
pub struct BlockState {
    pub block: Block,
    pub bonds: u64,
    pub unbonds: u64,
    pub validators: Vec<Validator>,
    pub supplies: Vec<Supply>,
    pub mint_limit: HashMap<String, u64>,
}

impl BlockState {
    pub fn new(
        block: Block,
        bonds: u64,
        unbonds: u64,
        validators: Vec<Validator>,
        supplies: Vec<Supply>,
        mint_limit: HashMap<String, u64>,
    ) -> Self {
        Self {
            block,
            bonds,
            unbonds,
            validators,
            supplies,
            mint_limit,
        }
    }

    pub fn consensus_validators(&self) -> Vec<Validator> {
        self.validators
            .iter()
            .filter(|validator| matches!(validator.state, ValidatorState::Consensus))
            .cloned()
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub blocks: Vec<BlockState>,
    size: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            size: 7200,
        }
    }
}

impl State {
    pub fn add_block(&mut self, block_state: BlockState) {
        if self.blocks.len() == self.size {
            self.blocks.remove(0);
        }
        self.blocks.push(block_state);
    }

    pub fn total_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn last_block(&self) -> BlockState {
        self.blocks.last().unwrap().clone()
    }

    pub fn prev_block(&self) -> BlockState {
        self.blocks[self.blocks.len() - 2].clone()
    }

    pub fn validators_with_voting_power(&self, fraction: f64) -> anyhow::Result<u64> {
        let block = self.last_block();

        let mut validators = block.validators.clone();
        validators.sort_by_key(|validator| validator.voting_power);
        validators.reverse();

        let total_voting_power = self.total_voting_power();
        let threshold_voting_power = (total_voting_power as f64 * fraction) as u64;
        let mut accumulated_voting_power = 0;

        for (index, validator) in validators.iter().enumerate() {
            if accumulated_voting_power >= threshold_voting_power {
                return Ok(index as u64);
            }
            accumulated_voting_power += validator.voting_power;
        }
        Err(anyhow!(
            "No validators can hold {} of the voting power",
            fraction
        ))
    }

    pub fn total_voting_power(&self) -> u64 {
        let block = self.last_block();

        block
            .validators
            .iter()
            .map(|validator| validator.voting_power)
            .sum()
    }

    pub fn get_all_transfers(&self) -> Vec<Transfer> {
        let block = self.last_block().block;

        let mut transfers = Vec::new();
        for tx in block
            .transactions
            .iter()
            .filter(|tx| tx.status.was_applied())
            .cloned()
            .collect::<Vec<Wrapper>>()
        {
            for inner in tx
                .inners
                .iter()
                .filter(|tx| tx.was_applied)
                .cloned()
                .collect::<Vec<Inner>>()
            {
                match &inner.kind {
                    InnerKind::Transfer(transfer) => {
                        let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
                        for (a, b) in &transfer.targets {
                            groups
                                .entry(a.token.to_string())
                                .or_default()
                                .push(b.amount().raw_amount().as_u64());
                        }
                        for (token, amounts) in groups {
                            let total: u64 = amounts.iter().sum();
                            transfers.push(Transfer {
                                height: block.height,
                                id: inner.id.clone(),
                                kind: TransferKind::Native,
                                token: token.clone(),
                                amount: total,
                                accepted: inner.was_applied,
                            });
                        }
                    }
                    InnerKind::IbcMsgTransfer(IbcMessage::Transfer(msg_transfer)) => {
                        if let Some(transfer) = &msg_transfer.transfer {
                            let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
                            for (a, b) in &transfer.targets {
                                groups
                                    .entry(a.token.to_string())
                                    .or_default()
                                    .push(b.amount().raw_amount().as_u64());
                            }
                            for (token, amounts) in groups {
                                let total: u64 = amounts.iter().sum();
                                transfers.push(Transfer {
                                    height: block.height,
                                    id: inner.id.clone(),
                                    kind: TransferKind::Ibc,
                                    token: token.clone(),
                                    amount: total,
                                    accepted: inner.was_applied,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        transfers
    }
}
