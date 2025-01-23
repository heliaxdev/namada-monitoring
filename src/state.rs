use anyhow::anyhow;

use crate::shared::namada::{Address, Block, Height, Transfer, Validator};

#[derive(Debug, Clone)]
pub struct State {
    //checksums: Checksums,
    block: Block,
    max_block_time_estimate: u64,
    total_supply_native: u64,
    native_token: Address,
    validators: Vec<Validator>,
    future_bonds: u64,
    future_unbonds: u64,
}

impl State {
    pub fn new(
        //checksums: Checksums,
        block: Block,
        native_token: Address,
        max_block_time_estimate: u64,
        total_supply_native: u64,
        validators: Vec<Validator>,
        future_bonds: u64,
        future_unbonds: u64,
    ) -> Self {
        Self {
            //checksums,
            block,
            native_token,
            max_block_time_estimate,
            total_supply_native,
            validators,
            future_bonds,
            future_unbonds,
        }
    }

    pub fn next_block_height(&self) -> Height {
        self.block.height + 1
    }

    pub fn max_next_block_timestamp_estimate(&self) -> i64 {
        self.block.timestamp + self.max_block_time_estimate as i64
    }

    pub fn get_max_block_time_estimate(&self) -> i64 {
        self.max_block_time_estimate as i64
    }

    pub fn get_last_block(&self) -> &Block {
        &self.block
    }

    pub fn get_total_supply(&self, token: &Address) -> Option<u64> {
        if token == &self.native_token {
            Some(self.total_supply_native)
        } else {
            None
        }
    }

    pub fn get_native_token(&self) -> &Address {
        &self.native_token
    }

    pub fn total_voting_power(&self) -> u64 {
        self.validators
            .iter()
            .map(|validator| validator.voting_power)
            .sum()
    }

    pub fn validators_with_voting_power(&self, fraction: f64) -> anyhow::Result<u64> {
        let mut validators = self.validators.clone();
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

    pub fn get_total_supply_native_token(&self) -> u64 {
        self.total_supply_native
    }

    pub fn get_future_bonds(&self) -> u64 {
        self.future_bonds
    }

    pub fn get_future_unbonds(&self) -> u64 {
        self.future_unbonds
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn get_epoch(&self) -> u64 {
        self.block.epoch
    }

    pub fn get_all_transfers(&self) -> Vec<Transfer> {
        self.block.get_all_transfers()
    }
}
