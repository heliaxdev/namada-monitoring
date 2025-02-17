use prometheus_exporter::prometheus::core::{AtomicF64, GenericCounterVec};

use prometheus_exporter::prometheus::{CounterVec, Opts, Registry};

use crate::state::State;

use super::MetricTrait;

pub struct Fees {
    /// fees counter
    fees_counter: GenericCounterVec<AtomicF64>,
}

impl MetricTrait for Fees {
    fn register(&self, registry: &Registry) -> anyhow::Result<()> {
        registry.register(Box::new(self.fees_counter.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {}

    fn update(&self, _pre_state: &State, post_state: &State) {
        let block = post_state.get_last_block();
        println!("Block height: {} tx number {} ", block.height, block.transactions.len());
        for tx in &block.transactions {
            let amount_per_gas = tx.fee.amount_per_gas_unit.parse::<f64>();
            let gas_limit = tx.fee.gas.parse::<f64>();

            let fee = match (amount_per_gas, gas_limit) {
                (Ok(amount_per_gas), Ok(gas_limit)) => amount_per_gas * gas_limit,
                _ => {println!("No fee"); continue},
            };

            self.fees_counter
                .with_label_values(&[&block.height.to_string(), &tx.fee.gas_token])
                .inc_by(fee);
        }
    }
}

impl Fees {
    pub fn default() -> Self {
        let fees_counter_opts =
            Opts::new("fees_counter", "Total fees paid per block and per token");
        let fees_counter = CounterVec::new(fees_counter_opts, &["height", "token"])
            .expect("unable to create int counter for transaction kinds");

        Self { fees_counter }
    }
}
