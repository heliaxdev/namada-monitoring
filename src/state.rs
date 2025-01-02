use std::num::NonZeroUsize;

use lru::LruCache;
use prometheus_exporter::prometheus::{
    core::{AtomicU64, GenericCounter}, Histogram, HistogramOpts, Registry
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
    pub checksums: Checksums,
    pub blocks: LruCache<Height, Block>,
    pub metrics: PrometheusMetrics,
}

#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    pub block_height_counter: GenericCounter<AtomicU64>,
    pub epoch_counter: GenericCounter<AtomicU64>,
    pub total_supply_native_token: GenericCounter<AtomicU64>,
    pub transaction_size: Histogram,
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

        let transaction_size_opts = HistogramOpts::new(
            "transaction_size_bytes",
            "The sizes of transactions in bytes"
        )
        .buckets(vec![
            10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0,
        ]);
        let transaction_size = Histogram::with_opts(transaction_size_opts).expect("unable to create histogram transaction sizes");;

        registry
            .register(Box::new(block_height_counter.clone()))
            .unwrap();
        registry.register(Box::new(epoch_counter.clone())).unwrap();
        registry
            .register(Box::new(total_supply_native_token.clone()))
            .unwrap();
        registry.register(Box::new(transaction_size.clone())).unwrap();
        
        Self {
            block_height_counter,
            epoch_counter,
            total_supply_native_token,
            transaction_size,
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
        Self {
            latest_block_height: Some(block_height),
            latest_epoch: None,
            latest_total_supply_native: None,
            checksums,
            blocks: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            metrics: PrometheusMetrics::new(),
        }
    }

    pub fn next_block_height(&self) -> Height {
        self.latest_block_height
            .map(|height| height + 1)
            .unwrap_or(1)
    }

    pub fn update(&mut self, block: Block, total_supply_native: u64) {
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

        for tx in &block.transactions {
            for inner in &tx.inners {
                self.metrics.transaction_size.observe(inner.kind.size() as f64);    
            }
        }

        self.blocks.put(block.height, block);
    }

    pub fn prometheus_registry(&self) -> Registry {
        self.metrics.registry.clone()
    }
}
