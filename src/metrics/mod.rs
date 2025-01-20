mod block_height_counter;
mod epoch_counter;
mod total_supply_native_token;
mod transaction_size;
mod voting_power;

use std::net::SocketAddr;

use block_height_counter::BlockHeightCounter;
use epoch_counter::EpochCounter;
use total_supply_native_token::TotalSupplyNativeToken;
use transaction_size::TransactionSize;
use voting_power::VotingPower;

use crate::{config::AppConfig, state::State};
use anyhow::{Context, Result};
use prometheus_exporter::prometheus::Registry;

pub enum Metrics {
    /// The latest block height recorded
    BlockHeightCounter(BlockHeightCounter),
    /// The latest epoch recorded
    EpochCounter(EpochCounter),
    /// The latest total supply native token recorded
    TotalNativeTokenSupply(TotalSupplyNativeToken),
    /// The latest transaction size recorded
    TransactionSize(TransactionSize),
    /// The latest voting power recorded in thirds
    VotingPower(VotingPower),
}

impl Metrics {
    pub fn reset(&self, state: &State) {
        match self {
            Metrics::BlockHeightCounter(counter) => counter.reset(state),
            Metrics::EpochCounter(counter) => counter.reset(state),
            Metrics::TotalNativeTokenSupply(counter) => counter.reset(state),
            Metrics::TransactionSize(counter) => counter.reset(state),
            Metrics::VotingPower(counter) => counter.reset(state),
        }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        match self {
            Metrics::BlockHeightCounter(counter) => counter.register(registry),
            Metrics::EpochCounter(counter) => counter.register(registry),
            Metrics::TotalNativeTokenSupply(counter) => counter.register(registry),
            Metrics::TransactionSize(counter) => counter.register(registry),
            Metrics::VotingPower(counter) => counter.register(registry),
        }
    }

    pub fn update(&self, pre_state: &State, post_state: &State) {
        match self {
            Metrics::BlockHeightCounter(counter) => counter.update(pre_state, post_state),
            Metrics::EpochCounter(counter) => counter.update(pre_state, post_state),
            Metrics::TotalNativeTokenSupply(counter) => counter.update(pre_state, post_state),
            Metrics::TransactionSize(counter) => counter.update(pre_state, post_state),
            Metrics::VotingPower(counter) => counter.update(pre_state, post_state),
        }
    }
}

pub struct MetricsCollection {
    metrics: Vec<Metrics>,
    registry: Registry,
}

impl MetricsCollection {
    pub fn new(_config: &AppConfig) -> Self {
        let registry = Registry::new_custom(None, None).expect("Failed to create registry");
        let metrics = vec![
            Metrics::BlockHeightCounter(BlockHeightCounter::default()),
            Metrics::EpochCounter(EpochCounter::default()),
            Metrics::TotalNativeTokenSupply(TotalSupplyNativeToken::default()),
            Metrics::TransactionSize(TransactionSize::default()),
            Metrics::VotingPower(VotingPower::default()),
        ];
        for metric in &metrics {
            metric.register(&registry).expect("Failed to register metric");
        }
        Self { metrics, registry }        
    }
    pub fn start_exporter(&self, port: u64) -> anyhow::Result<()> {
        let addr_raw = format!("0.0.0.0:{}", port);
        let addr: SocketAddr = addr_raw.parse().context("can not parse listen addr")?;

        let mut builder = prometheus_exporter::Builder::new(addr);
        builder.with_registry(self.registry.clone());
        builder.start().context("can not start exporter")?;

        Ok(())
    }

    pub fn reset(&self, state: &State) {
        for metric in &self.metrics {
            metric.reset(state);
        }
    }


    pub fn update(&self, pre_state: &State, post_state: &State) {
        for metric in &self.metrics {
            metric.update(pre_state, post_state);
        }
    }
}
