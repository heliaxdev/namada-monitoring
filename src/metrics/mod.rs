mod alert;
mod block_height;
mod block_time;
mod bonds;
mod epoch;
mod fees;
mod signatures;
pub mod slashes;
mod total_supply_native_token;
mod transactions;
mod transfers;
mod validator;
mod voting_power;

use std::{collections::HashMap, net::SocketAddr};

use block_height::BlockHeight;
use block_time::BlockTime;
use bonds::Bonds;
use epoch::Epoch;
use fees::Fees;
use signatures::Signatures;
use slashes::Slashes;
use total_supply_native_token::TotalSupplyNativeToken;
use transactions::Transactions;
use transfers::Transfers;
use validator::ValidatorState;
use voting_power::VotingPower;

use crate::{
    config::AppConfig,
    state::State,
};
use anyhow::{Context, Result};
use prometheus_exporter::prometheus::Registry;

pub trait MetricTrait {
    fn register(&self, registry: &Registry) -> Result<()>;
    fn reset(&self, state: &State);
    fn update(&self, pre_state: &State, post_state: &State);
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
            // TODO: check that it is enabled in the config
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
            Box::<BlockHeight>::default() as Box<dyn MetricTrait>,
            Box::<Bonds>::default() as Box<dyn MetricTrait>,
            Box::<Epoch>::default() as Box<dyn MetricTrait>,
            Box::<TotalSupplyNativeToken>::default() as Box<dyn MetricTrait>,
            Box::<Transactions>::default() as Box<dyn MetricTrait>,
            Box::<Transfers>::default() as Box<dyn MetricTrait>,
            Box::<VotingPower>::default() as Box<dyn MetricTrait>,
            Box::<BlockTime>::default() as Box<dyn MetricTrait>,
            Box::<Fees>::default() as Box<dyn MetricTrait>,
            Box::<Signatures>::default() as Box<dyn MetricTrait>,
            Box::<Slashes>::default() as Box<dyn MetricTrait>,
            Box::<ValidatorState>::default() as Box<dyn MetricTrait>,
            Box::new(alert::Alert::new(config)) as Box<dyn MetricTrait>,
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
    pub fn start_exporter_with(&self, state: &State) -> anyhow::Result<()> {
        self.reset(state);
        self.start_exporter()
            .context("can not start exporter with state")?;
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
