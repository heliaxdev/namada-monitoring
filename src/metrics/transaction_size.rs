use crate::state::State;
use anyhow::Result;
use prometheus_exporter::prometheus::{Histogram, HistogramOpts, Registry};

pub struct TransactionSize {
    /// Histogram of recorded transaction sizes
    transaction_size: Histogram,
}

impl TransactionSize {
    pub fn default() -> Self {
        let transaction_size_opts = HistogramOpts::new(
            "transaction_size_bytes",
            "The sizes of transactions in bytes",
        )
        .buckets(vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0]);
        let transaction_size = Histogram::with_opts(transaction_size_opts)
            .expect("unable to create histogram transaction sizes");
        Self { transaction_size }
    }

    pub fn register(&self, registry: &Registry) -> Result<()> {
        registry.register(Box::new(self.transaction_size.clone()))?;
        Ok(())
    }

    pub fn reset(&self, _state: &State) {}

    pub fn update(&self, _pre_state: &State, post_state: &State) {
        // update transaction size metrics
        for tx in &post_state.get_last_block().transactions {
            //self.transaction_inner_size.observe(tx.inners.len() as f64);
            for inner in &tx.inners {
                // let inner_kind = inner.kind.to_string();
                // self.transaction_kind
                //     .with_label_values(&[
                //         &inner_kind,
                //         &post_state.block.epoch.to_string(),
                //         &post_state.block.height.to_string(),
                //     ])
                //     .inc();
                self.transaction_size.observe(inner.size as f64);
            }
        }
    }
}
