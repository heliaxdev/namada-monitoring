use std::num::NonZeroUsize;

use lru::LruCache;
use prometheus_exporter::prometheus::{
    core::{AtomicU64, GenericCounter},
    Registry,
};

use crate::shared::{
    checksums::Checksums,
    namada::{Block, Height},
};

#[derive(Debug, Clone)]
pub struct State {
    pub latest_block_height: Option<u64>,
    pub latest_epoch: Option<u64>,
    pub latest_total_supply_native: Option<u64>,
    pub latest_max_block_time_estimate: Option<u64>,
    pub checksums: Checksums,
    pub blocks: LruCache<Height, Block>,
    pub metrics: PrometheusMetrics,
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

    pub fn increase_block_height(&self) {
        self.block_height_counter.inc();
    }

    pub fn increase_epoch(&self) {
        self.epoch_counter.inc();
    }
}

impl State {
    pub fn new(checksums: Checksums, block_height: u64) -> Self {
        let mut new_state = Self {
            latest_block_height: Some(block_height),
            latest_epoch: None,
            latest_total_supply_native: None,
            latest_max_block_time_estimate: None,
            checksums,
            blocks: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            metrics: PrometheusMetrics::new(),
        };
        new_state.reset_metrics();
        new_state
    }

    /// Initializes/resets metrics to current state
    pub fn reset_metrics(&mut self) {
        self.metrics.block_height_counter.reset();
        self.metrics.epoch_counter.reset();
        self.metrics.total_supply_native_token.reset();
        self.metrics.block_height_counter.inc_by(self.latest_block_height.unwrap_or(0));
        self.metrics.epoch_counter.inc_by(self.latest_epoch.unwrap_or(0));
        self.metrics.total_supply_native_token.inc_by(self.latest_total_supply_native.unwrap_or(0));
    }

    pub fn next_block_height(&self) -> Height {
        self.latest_block_height
            .map(|height| height + 1)
            .unwrap_or(1)
    }

    pub fn get_last_block(&mut self) -> Option<&Block> {
        self.latest_block_height.and_then(|height| self.blocks.get(&height))
    }

    pub fn update(&mut self, block: Block, total_supply_native: u64, max_block_time_estimate: u64) {
        if let Some(height) = self.latest_block_height {
            self.metrics
                .block_height_counter
                .inc_by(block.height - height);
        } else {
            self.metrics.block_height_counter.inc_by(block.height);
        }
        self.latest_block_height = Some(block.height);

        if let Some(epoch) = self.latest_epoch {
            self.metrics
                .block_height_counter
                .inc_by(block.epoch - epoch);
        } else {
            self.metrics.block_height_counter.inc_by(block.epoch);
        }
        self.latest_epoch = Some(block.epoch);

        if let Some(total_supply) = self.latest_total_supply_native {
            self.metrics
                .total_supply_native_token
                .inc_by(total_supply_native - total_supply);
        } else {
            self.metrics
                .total_supply_native_token
                .inc_by(total_supply_native);
        }
        self.latest_total_supply_native = Some(total_supply_native);
        self.latest_max_block_time_estimate = Some(max_block_time_estimate);
        self.blocks.put(block.height, block);
    }

    pub fn prometheus_registry(&self) -> Registry {
        self.metrics.registry.clone()
    }
}
