mod block_height_counter;
mod bonds;
mod epoch_counter;
mod total_supply_native_token;
mod transactions;
mod transfers;
mod voting_power;
mod block_time;

use std::{collections::HashMap, net::SocketAddr};

use block_height_counter::BlockHeightCounter;
use bonds::Bonds;
use epoch_counter::EpochCounter;
use total_supply_native_token::TotalSupplyNativeToken;
use transactions::Transactions;
use transfers::Transfers;
use voting_power::VotingPower;
use block_time::BlockTime;

use crate::{config::AppConfig, state::State};
use anyhow::{Context, Result};
use prometheus_exporter::prometheus::Registry;

pub trait MetricTrait {
    fn register(&self, registry: &Registry) -> Result<()>;
    fn reset(&self, state: &State);
    fn update(&self, pre_state: &State, post_state: &State);
}

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
    /// The time spent processing block
    BlockTime(BlockTime),
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
            Metrics::BlockTime(counter) => counter.reset(state),
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
            Metrics::BlockTime(counter) => counter.register(registry),
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
            Metrics::BlockTime(counter) => counter.update(pre_state, post_state),
        }
    }
}

pub struct MetricsExporter {
    registry: Registry,
    metrics: Vec<Box<dyn MetricTrait>>,
    port: u64,
}

impl MetricsExporter {
    pub fn new(config: &AppConfig, metrics: Vec<Box<dyn MetricTrait>>) -> Self {
        let port = config.prometheus_port;

        let registry = Registry::new_custom(
            Some("namada".to_string()),
            Some(HashMap::from_iter([(
                "chain_id".to_string(),
                config.chain_id.as_ref().unwrap().to_string(),
            )])),
        )
        .expect("Failed to create registry");

        for metric in &metrics {
            metric
                .register(&registry)
                .expect("Failed to register metric");
        }
        Self {
            port,
            metrics,
            registry,
        }
    }

    pub fn default_metrics(config: &AppConfig) -> Self {
        let metrics = vec![
            Box::new(BlockHeightCounter::default()) as Box<dyn MetricTrait>,
            Box::new(Bonds::default()) as Box<dyn MetricTrait>,
            Box::new(EpochCounter::default()) as Box<dyn MetricTrait>,
            Box::new(TotalSupplyNativeToken::default()) as Box<dyn MetricTrait>,
            Box::new(Transactions::default()) as Box<dyn MetricTrait>,
            Box::new(Transfers::default()) as Box<dyn MetricTrait>,
            Box::new(VotingPower::default()) as Box<dyn MetricTrait>,
        ];

        Self::new(config, metrics)
    }

    pub fn start_exporter(&self) -> anyhow::Result<()> {
        let addr_raw = format!("0.0.0.0:{}", self.port);
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
