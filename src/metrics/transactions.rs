use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{core::{AtomicU64, GenericCounterVec}, Histogram, HistogramOpts, IntCounterVec, Opts, Registry};

pub struct Transactions {
    /// inner transactions count in the batch histogram
    transaction_batch_size: Histogram,
    /// inner transaction kind counter by epoch
    transaction_kind: GenericCounterVec<AtomicU64>,
}

impl Transactions {
    pub fn default() -> Self {
        let transaction_batch_size_opts = HistogramOpts::new(
            "transaction_batch_size",
            "The number of inner transactions in the batch",
        )
        .buckets(vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0]);
        let transaction_batch_size = Histogram::with_opts(transaction_batch_size_opts)
            .expect("unable to create histogram transaction batch size");

        let transaction_kind_opts =
            Opts::new("transaction_kind", "Transaction kind count per epoch");
        let transaction_kind =
            IntCounterVec::new(transaction_kind_opts, &["kind", "epoch" /* , "height"*/])
                .expect("unable to create int counter for transaction kinds");


        Self { transaction_batch_size, transaction_kind }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.transaction_batch_size.clone()))?;
        registry.register(Box::new(self.transaction_kind.clone()))?;
        Ok(())
    }

    pub fn reset(&self, _state: &State) {}

    pub fn update(&self, _pre_state: &State, post_state: &State) {
        // update transaction size metrics
        for tx in &post_state.get_last_block().transactions {
            self.transaction_batch_size.observe(tx.inners.len() as f64);
            //self.transaction_inner_size.observe(tx.inners.len() as f64);
            for inner in &tx.inners {
                let inner_kind = inner.kind.to_string();
                self.transaction_kind
                    .with_label_values(&[
                        &inner_kind,
                        &post_state.get_epoch().to_string(),
                        // &post_state.get_block().height.to_string(),
                    ])
                    .inc();                
            }
        }
    }
}
