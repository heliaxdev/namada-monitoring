use std::num::NonZeroUsize;

use lru::LruCache;
use prometheus_exporter::prometheus::{
    core::{AtomicU64, GenericCounter, GenericCounterVec, GenericGauge},
    GaugeVec, Histogram, HistogramOpts, IntCounterVec, Opts, Registry,
};

use crate::shared::{
    checksums::Checksums,
    namada::{Block, Height, Validator},
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
    pub one_third_threshold: GenericGauge<AtomicU64>,
    pub two_third_threshold: GenericGauge<AtomicU64>,
    pub transaction_size: Histogram,
    pub bonds_per_epoch: GaugeVec,
    pub unbonds_per_epoch: GaugeVec,
    pub transaction_kind: GenericCounterVec<AtomicU64>,
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

        let one_third_threshold = GenericGauge::<AtomicU64>::new(
            "one_third_threshold",
            "the number of validators to reach 1/3 of the voting power",
        )
        .expect("unable to create counter one third threshold");

        let two_third_threshold = GenericGauge::<AtomicU64>::new(
            "two_third_threshold",
            "the number of validators to reach 2/3 of the voting power",
        )
        .expect("unable to create counter two third threshold");

        let total_supply_native_token = GenericCounter::<AtomicU64>::new(
            "total_supply_native_token",
            "the latest total supply native token recorded",
        )
        .expect("unable to create counter total supply");

        let transaction_size_opts = HistogramOpts::new(
            "transaction_batch_size",
            "The number of inner transactions in a batch",
        )
        .buckets(vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0]);
        let transaction_size = Histogram::with_opts(transaction_size_opts)
            .expect("unable to create histogram transaction sizes");

        let bonds_per_epoch_opts = Opts::new("bonds_per_epoch", "Total bonds per epoch");
        let bonds_per_epoch = GaugeVec::new(bonds_per_epoch_opts, &["epoch"])
            .expect("unable to create histogram transaction sizes");

        let unbonds_per_epoch_opts = Opts::new("unbonds_per_epoch", "Total unbonds per epoch");
        let unbonds_per_epoch = GaugeVec::new(unbonds_per_epoch_opts, &["epoch"])
            .expect("unable to create histogram transaction sizes");

        let transaction_kind_opts =
            Opts::new("transaction_kind", "Total transaction per transaction kind");
        let transaction_kind =
            IntCounterVec::new(transaction_kind_opts, &["kind", "epoch", "height"])
                .expect("unable to create histogram transaction sizes");

        registry
            .register(Box::new(block_height_counter.clone()))
            .unwrap();
        registry.register(Box::new(epoch_counter.clone())).unwrap();
        registry
            .register(Box::new(total_supply_native_token.clone()))
            .unwrap();
        registry
            .register(Box::new(one_third_threshold.clone()))
            .unwrap();
        registry
            .register(Box::new(two_third_threshold.clone()))
            .unwrap();
        registry
            .register(Box::new(transaction_size.clone()))
            .unwrap();
        registry
            .register(Box::new(bonds_per_epoch.clone()))
            .unwrap();
        registry
            .register(Box::new(unbonds_per_epoch.clone()))
            .unwrap();
        registry
            .register(Box::new(transaction_kind.clone()))
            .unwrap();

        Self {
            block_height_counter,
            epoch_counter,
            total_supply_native_token,
            one_third_threshold,
            two_third_threshold,
            transaction_size,
            bonds_per_epoch,
            unbonds_per_epoch,
            transaction_kind,
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

    pub fn update(
        &mut self,
        block: Block,
        total_supply_native: u64,
        future_bonds: u64,
        future_unbonds: u64,
        mut validators: Vec<Validator>,
    ) {
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
            self.metrics.epoch_counter.inc_by(block.epoch);
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
            self.metrics
                .transaction_size
                .observe(tx.inners.len() as f64);
        }

        for tx in &block.transactions {
            for inner in &tx.inners {
                let inner_kind = inner.kind.to_string();

                self.metrics
                    .transaction_kind
                    .with_label_values(&[
                        &inner_kind,
                        &block.epoch.to_string(),
                        &block.height.to_string(),
                    ])
                    .inc();
            }
        }

        validators.sort_by_key(|validator| validator.voting_power);
        validators.reverse();
        let total_voting_power = validators
            .iter()
            .map(|validator| validator.voting_power)
            .sum::<u64>();
        let one_third_voting_power = total_voting_power / 3;
        let two_third_voting_power = total_voting_power * 2 / 3;
        let (one_third_threshold, _) = validators.iter().fold((0, 0), |(index, acc), validator| {
            if acc >= one_third_voting_power {
                (index, acc)
            } else {
                (index + 1, acc + validator.voting_power)
            }
        });

        let (two_third_threshold, _) = validators.iter().fold((0, 0), |(index, acc), validator| {
            if acc >= two_third_voting_power {
                (index, acc)
            } else {
                (index + 1, acc + validator.voting_power)
            }
        });
        self.metrics.one_third_threshold.set(one_third_threshold);
        self.metrics.two_third_threshold.set(two_third_threshold);

        self.metrics
            .bonds_per_epoch
            .with_label_values(&[&(block.epoch + 1).to_string()])
            .set(future_bonds as f64);
        self.metrics
            .unbonds_per_epoch
            .with_label_values(&[&(block.epoch + 1).to_string()])
            .set(future_unbonds as f64);

        self.blocks.put(block.height, block);
    }

    pub fn prometheus_registry(&self) -> Registry {
        self.metrics.registry.clone()
    }
}
