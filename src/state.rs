use anyhow::{anyhow, Context};
use std::net::SocketAddr;

//use lru::LruCache;
use prometheus_exporter::prometheus::{
    core::{AtomicU64, GenericCounter},
    Registry,
};

use crate::shared::{
    checksums::Checksums,
    namada::{Address, Block, Height},
};

#[derive(Debug, Clone)]
pub struct State {
    block: Block,
    max_block_time_estimate: u64,
    total_supply_native_token: u64,
    checksums: Checksums,
    native_token: Address,
    //blocks: LruCache<Height, Block>,
}

#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    /// The latest block height recorded
    pub block_height_counter: GenericCounter<AtomicU64>,
    /// The latest epoch recorded
    pub epoch_counter: GenericCounter<AtomicU64>,
    /// The latest total supply native token recorded
    pub total_supply_native_token: GenericCounter<AtomicU64>,
    registry: Registry,
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new_custom(None, None).expect("Failed to create registry");

        let block_height_counter =
            GenericCounter::<AtomicU64>::new("block_height", "the latest block height recorded")
                .expect("unable to create counter block_height");

        let epoch_counter = GenericCounter::<AtomicU64>::new("epoch", "the latest epoch recorded")
            .expect("unable to create counter epoch");

        let total_supply_native_token = GenericCounter::<AtomicU64>::new(
            "total_supply_native_token",
            "the latest total supply native token recorded",
        )
        .expect("unable to create counter total supply");

        registry
            .register(Box::new(block_height_counter.clone()))
            .unwrap();
        registry.register(Box::new(epoch_counter.clone())).unwrap();
        registry
            .register(Box::new(total_supply_native_token.clone()))
            .unwrap();

        Self {
            block_height_counter,
            epoch_counter,
            total_supply_native_token,
            registry,
        }
    }

    pub fn update(&self, pre_state: &State, post_state: &State) {
        // consider the differences between pre and post state and update the metrics
        self.block_height_counter
            .inc_by(post_state.block.height - pre_state.block.height);
        self.epoch_counter
            .inc_by(post_state.block.epoch - pre_state.block.epoch);
        self.total_supply_native_token
            .inc_by(post_state.total_supply_native_token - pre_state.total_supply_native_token);
    }

    pub fn start_exporter(&self, port: u64) -> anyhow::Result<()> {
        let addr_raw = format!("0.0.0.0:{}", port);
        let addr: SocketAddr = addr_raw.parse().context("can not parse listen addr")?;

        let mut builder = prometheus_exporter::Builder::new(addr);
        builder.with_registry(self.registry.clone());
        builder.start().context("can not start exporter")?;

        Ok(())
    }

    // resets metrics to current state
    pub fn reset_metrics(&self, state: &State) {
        self.block_height_counter.reset();
        self.epoch_counter.reset();
        self.total_supply_native_token.reset();

        self.block_height_counter.inc_by(state.block.height);
        self.epoch_counter.inc_by(state.block.epoch);
        self.total_supply_native_token
            .inc_by(state.total_supply_native_token);
    }
}

impl State {
    pub fn new(
        block: Block,
        checksums: Checksums,
        native_token: Address,
        max_block_time_estimate: u64,
        total_supply_native_token: u64,
    ) -> Self {
        let mut new_state = Self {
            block,
            total_supply_native_token,
            max_block_time_estimate,
            checksums,
            native_token,
            //blocks: LruCache::new(NonZeroUsize::new(1024).unwrap()),
        };
        new_state
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

    // pub fn get_block(&mut self, height: Height) -> Option<&Block> {
    //     if height == self.block.height {
    //         Some(&self.block)
    //     } else {
    //         self.blocks.get(&height)
    //     }
    // }

    pub fn get_total_supply(&self, token: &Address) -> Option<u64> {
        if token == &self.native_token {
            Some(self.total_supply_native_token)
        } else {
            None
        }
    }

    pub fn get_native_token(&self) -> &Address {
        &self.native_token
    }
}
