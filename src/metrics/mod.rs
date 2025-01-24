mod block_height_counter;
mod bonds;
mod epoch_counter;
mod total_supply_native_token;
mod transactions;
mod transfers;
mod voting_power;

use std::{collections::HashMap, net::SocketAddr};

use block_height_counter::BlockHeightCounter;
use bonds::Bonds;
use epoch_counter::EpochCounter;
use total_supply_native_token::TotalSupplyNativeToken;
use transactions::Transactions;
use transfers::Transfers;
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
    /// Various transaction metrics
    Transactions(Transactions),
    /// The latest voting power recorded in thirds
    VotingPower(VotingPower),
    /// The latest bounds/unbounds count
    Bounds(Bonds),
    /// Total transfers by epoch and token
    Transfers(Transfers),
}

// FIXME this could be a trait
impl Metrics {
    pub fn reset(&self, state: &State) {
        match self {
            Metrics::BlockHeightCounter(counter) => counter.reset(state),
            Metrics::EpochCounter(counter) => counter.reset(state),
            Metrics::TotalNativeTokenSupply(counter) => counter.reset(state),
            Metrics::Transactions(counter) => counter.reset(state),
            Metrics::VotingPower(counter) => counter.reset(state),
            Metrics::Bounds(counter) => counter.reset(state),
            Metrics::Transfers(counter) => counter.reset(state),
        }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        match self {
            Metrics::BlockHeightCounter(counter) => counter.register(registry),
            Metrics::EpochCounter(counter) => counter.register(registry),
            Metrics::TotalNativeTokenSupply(counter) => counter.register(registry),
            Metrics::Transactions(counter) => counter.register(registry),
            Metrics::VotingPower(counter) => counter.register(registry),
            Metrics::Bounds(counter) => counter.register(registry),
            Metrics::Transfers(counter) => counter.register(registry),
        }
    }

    pub fn update(&self, pre_state: &State, post_state: &State) {
        match self {
            Metrics::BlockHeightCounter(counter) => counter.update(pre_state, post_state),
            Metrics::EpochCounter(counter) => counter.update(pre_state, post_state),
            Metrics::TotalNativeTokenSupply(counter) => counter.update(pre_state, post_state),
            Metrics::Transactions(counter) => counter.update(pre_state, post_state),
            Metrics::VotingPower(counter) => counter.update(pre_state, post_state),
            Metrics::Bounds(counter) => counter.update(pre_state, post_state),
            Metrics::Transfers(counter) => counter.update(pre_state, post_state),
        }
    }
}

pub struct MetricsCollection {
    metrics: Vec<Metrics>,
    registry: Registry,
}

impl MetricsCollection {
    pub fn new(config: &AppConfig) -> Self {
        let registry = Registry::new_custom(
            Some("namada".to_string()),
            Some(HashMap::from_iter([(
                "chain_id".to_string(),
                config.chain_id.as_ref().unwrap().to_string(),
            )])),
        )
        .expect("Failed to create registry");

        let metrics = vec![
            Metrics::BlockHeightCounter(BlockHeightCounter::default()),
            Metrics::EpochCounter(EpochCounter::default()),
            Metrics::TotalNativeTokenSupply(TotalSupplyNativeToken::default()),
            Metrics::Transactions(Transactions::default()),
            Metrics::VotingPower(VotingPower::default()),
            Metrics::Bounds(Bonds::default()),
            Metrics::Transfers(Transfers::default()),
        ];
        for metric in &metrics {
            metric
                .register(&registry)
                .expect("Failed to register metric");
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
