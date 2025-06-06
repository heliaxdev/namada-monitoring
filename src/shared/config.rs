use serde::Deserialize;

use super::block_explorer::BlockExplorer;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub block_explorer: BlockExplorer,
    pub chain: Chain,
    pub pos: Pos,
    pub tx: Tx,
    pub ibcs: Vec<Ibc>,
    pub fees: Vec<FeeThreshold>,
    pub slack: Option<SlackAlertConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeeThreshold {
    pub alias: String,
    pub token: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Chain {
    pub block_time: f64,
    pub block_time_max_deviation: f64,
    pub halt_threshold: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pos {
    pub mininimum_one_third_validators: u64,
    pub mininimum_two_third_validators: u64,
    pub bond_increase_threshold: f64,
    pub unbond_increase_threshold: f64,
    pub consensus_threshold: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tx {
    pub threshold_batch: u64,
    pub threshold_sections: u64,
    pub gas_limit_threshold: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ibc {
    pub alias: String,
    pub channel: u64,
    pub connection: u64,
    pub client_id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackAlertConfig {
    pub slack_webhook: String,
    pub channel: String,
}

impl Config {
    pub fn tokens(&self) -> Vec<(String, String)> {
        self.fees
            .iter()
            .map(|fee| (fee.alias.clone(), fee.token.clone()))
            .collect()
    }

    pub fn ibcs(&self) -> Vec<(String, u64, u64, u64)> {
        self.ibcs
            .iter()
            .map(|ibc| {
                (
                    ibc.alias.clone(),
                    ibc.channel,
                    ibc.connection,
                    ibc.client_id,
                )
            })
            .collect()
    }
}
