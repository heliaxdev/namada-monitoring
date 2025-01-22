use anyhow::anyhow;

use crate::shared::{
    checksums::Checksums,
    namada::{Address, Block, Height, Transfer, Validator},
};

#[derive(Debug, Clone)]
pub struct State {
    block: Block,
    max_block_time_estimate: u64,
    total_supply_native: u64,
    checksums: Checksums,
    native_token: Address,
    validators: Vec<Validator>,
    future_bonds: u64,
    future_unbonds: u64,
}

// #[derive(Debug, Clone)]
// pub struct PrometheusMetrics {
////     /// The latest total supply native token recorded
//    pub total_supply_native_token: GenericCounter<AtomicU64>,
//     pub transaction_size: Histogram,
//     pub transaction_inner_size: Histogram,
//     pub transaction_kind: GenericCounterVec<AtomicU64>,
//     pub one_third_threshold: GaugeVec,
//     pub two_third_threshold: GaugeVec,
//     pub bonds_per_epoch: GaugeVec,
//     pub unbonds_per_epoch: GaugeVec,
//     registry: Registry,
// }

// impl Default for PrometheusMetrics {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl PrometheusMetrics {
//     pub fn new() -> Self {
//         let registry = Registry::new_custom(None, None).expect("Failed to create registry");
//         let block_height_counter =
//             GenericCounter::<AtomicU64>::new("block_height", "the latest block height recorded")
//                 .expect("unable to create counter block_height");

//         let epoch_counter = GenericCounter::<AtomicU64>::new("epoch", "the latest epoch recorded")
//             .expect("unable to create counter epoch");

//         let one_third_threshold_opts = Opts::new(
//             "one_third_threshold",
//             "The number of validators to reach 1/3 of the voting power",
//         );
//         let one_third_threshold = GaugeVec::new(one_third_threshold_opts, &["epoch"])
//             .expect("unable to create counter one third threshold");

//         let two_third_threshold_opts = Opts::new(
//             "two_third_threshold",
//             "The number of validators to reach 2/3 of the voting power",
//         );
//         let two_third_threshold = GaugeVec::new(two_third_threshold_opts, &["epoch"])
//             .expect("unable to create counter two third threshold");

//         let total_supply_native_token = GenericCounter::<AtomicU64>::new(
//             "total_supply_native_token",
//             "the latest total supply native token recorded",
//         )
//         .expect("unable to create counter total supply");

//         let transaction_size_opts = HistogramOpts::new(
//             "transaction_size_bytes",
//             "The sizes of transactions in bytes",
//         )
//         .buckets(vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0]);
//         let transaction_size = Histogram::with_opts(transaction_size_opts)
//             .expect("unable to create histogram transaction sizes");

//         let transaction_inner_size_opts =
//             HistogramOpts::new("transaction_inners", "The number of inner tx for a wrapper")
//                 .buckets(vec![2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0]);

//         let transaction_inner_size = Histogram::with_opts(transaction_inner_size_opts)
//             .expect("unable to create histogram transaction sizes");

//         let bonds_per_epoch_opts = Opts::new("bonds_per_epoch", "Total bonds per epoch");
//         let bonds_per_epoch = GaugeVec::new(bonds_per_epoch_opts, &["epoch"])
//             .expect("unable to create histogram transaction sizes");

//         let unbonds_per_epoch_opts = Opts::new("unbonds_per_epoch", "Total unbonds per epoch");
//         let unbonds_per_epoch = GaugeVec::new(unbonds_per_epoch_opts, &["epoch"])
//             .expect("unable to create histogram transaction sizes");

//         let transaction_kind_opts =
//             Opts::new("transaction_kind", "Total transaction per transaction kind");
//         let transaction_kind =
//             IntCounterVec::new(transaction_kind_opts, &["kind", "epoch", "height"])
//                 .expect("unable to create histogram transaction sizes");

//         // Register metrics
//         registry
//             .register(Box::new(block_height_counter.clone()))
//             .unwrap();
//         registry.register(Box::new(epoch_counter.clone())).unwrap();
//         registry
//             .register(Box::new(total_supply_native_token.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(transaction_size.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(transaction_inner_size.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(one_third_threshold.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(two_third_threshold.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(bonds_per_epoch.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(unbonds_per_epoch.clone()))
//             .unwrap();
//         registry
//             .register(Box::new(transaction_kind.clone()))
//             .unwrap();

//         Self {
//             block_height_counter,
//             epoch_counter,
//             total_supply_native_token,
//             transaction_size,
//             transaction_inner_size,
//             one_third_threshold,
//             two_third_threshold,
//             bonds_per_epoch,
//             unbonds_per_epoch,
//             transaction_kind,
//             registry,
//         }
//     }

//     pub fn update(&self, pre_state: &State, post_state: &State) {
//         // update block height
//         self.block_height_counter
//             .inc_by(post_state.block.height - pre_state.block.height);
//         // update epoch
//         self.epoch_counter
//             .inc_by(post_state.block.epoch - pre_state.block.epoch);
//         // update total supply
//         self.total_supply_native_token
//             .inc_by(post_state.total_supply_native - pre_state.total_supply_native);

//         // update transaction size metrics
//         for tx in &post_state.block.transactions {
//             self.transaction_inner_size.observe(tx.inners.len() as f64);
//             for inner in &tx.inners {
//                 let inner_kind = inner.kind.to_string();
//                 self.transaction_kind
//                     .with_label_values(&[
//                         &inner_kind,
//                         &post_state.block.epoch.to_string(),
//                         &post_state.block.height.to_string(),
//                     ])
//                     .inc();

//                 self.transaction_size.observe(inner.kind.size() as f64);
//             }
//         }

//         self.one_third_threshold
//             .with_label_values(&[&post_state.block.epoch.to_string()])
//             .set(
//                 post_state
//                     .validators_with_voting_power(1.0 / 3.0)
//                     .unwrap_or_default() as f64,
//             );
//         self.two_third_threshold
//             .with_label_values(&[&post_state.block.epoch.to_string()])
//             .set(
//                 post_state
//                     .validators_with_voting_power(2.0 / 3.0)
//                     .unwrap_or_default() as f64,
//             );

//         self.bonds_per_epoch
//             .with_label_values(&[&(post_state.block.epoch + 1).to_string()])
//             .set(post_state.future_bonds as f64);
//         self.unbonds_per_epoch
//             .with_label_values(&[&(post_state.block.epoch + 1).to_string()])
//             .set(post_state.future_unbonds as f64);
//     }


//     // resets metrics to current state
//     pub fn reset_metrics(&self, state: &State) {
//         self.block_height_counter.reset();
//         self.epoch_counter.reset();
//         self.total_supply_native_token.reset();

//         self.block_height_counter.inc_by(state.block.height);
//         self.epoch_counter.inc_by(state.block.epoch);
//         self.total_supply_native_token
//             .inc_by(state.total_supply_native);
//     }
// }

impl State {
    pub fn new(
        block: Block,
        checksums: Checksums,
        native_token: Address,
        max_block_time_estimate: u64,
        total_supply_native: u64,
        validators: Vec<Validator>,
        future_bonds: u64,
        future_unbonds: u64,
    ) -> Self {
        Self {
            block,
            checksums,
            native_token,
            max_block_time_estimate,
            total_supply_native,
            validators,
            future_bonds,
            future_unbonds,
        }
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

    pub fn get_total_supply(&self, token: &Address) -> Option<u64> {
        if token == &self.native_token {
            Some(self.total_supply_native)
        } else {
            None
        }
    }

    pub fn get_native_token(&self) -> &Address {
        &self.native_token
    }

    pub fn total_voting_power(&self) -> u64 {
        self.validators
            .iter()
            .map(|validator| validator.voting_power)
            .sum()
    }

    pub fn validators_with_voting_power(&self, fraction: f64) -> anyhow::Result<u64> {
        let mut validators = self.validators.clone();
        validators.sort_by_key(|validator| validator.voting_power);
        validators.reverse();

        let total_voting_power = self.total_voting_power();
        let threshold_voting_power = (total_voting_power as f64 * fraction) as u64;
        let mut accumulated_voting_power = 0;

        for (index, validator) in validators.iter().enumerate() {
            if accumulated_voting_power >= threshold_voting_power {
                return Ok(index as u64);
            }
            accumulated_voting_power += validator.voting_power;
        }
        Err(anyhow!(
            "No validators can hold {} of the voting power",
            fraction
        ))
    }

    pub fn get_total_supply_native_token(&self) -> u64 {
        self.total_supply_native
    }

    pub fn get_future_bonds(&self) -> u64 {
        self.future_bonds
    }
    
    pub fn get_future_unbonds(&self) -> u64 {
        self.future_unbonds
    }

    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn get_epoch(&self) -> u64 {
        self.block.epoch
    }

    pub fn get_all_transfers(&self) -> Vec<Transfer> {
        self.block.get_all_transfers()
    }

}
