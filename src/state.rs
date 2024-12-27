use std::{collections::HashMap, num::NonZeroUsize};

use lru::LruCache;
use prometheus_exporter::prometheus::{
    core::{AtomicF64, GenericCounter, GenericGauge},
    register_counter, register_counter_with_registry, register_gauge, Registry,
};

use crate::shared::{
    checksums::Checksums,
    namada::{Block, Height},
};

#[derive(Debug, Clone)]
pub struct State {
    pub latest_block_height: Option<u64>,
    pub latest_epoch: Option<u64>,
    pub checksums: Checksums,
    pub blocks: LruCache<Height, Block>,
    pub metrics: PrometheusMetrics,
}

#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    pub block_height_counter: GenericCounter<AtomicF64>,
    pub epoch_counter: GenericCounter<AtomicF64>,
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        Self {
            block_height_counter: register_counter!(
                "block_height",
                "the latest block height recorded",
            )
            .expect("unable to create counter block_height"),
            epoch_counter: register_counter!("epoch", "the latest epoch recorded",)
                .expect("unable to create counter epoch"),
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
    pub fn new(checksums: Checksums) -> Self {
        Self {
            latest_block_height: None,
            latest_epoch: None,
            checksums,
            blocks: LruCache::new(NonZeroUsize::new(1024).unwrap()),
            metrics: PrometheusMetrics::new(),
        }
    }
}
