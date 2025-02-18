use crate::state::State;
/// ## Fees Metric. (fees_counter)
/// This metric tracks the total transaction fees paid per block and per token. It helps monitor the gas costs of transactions on
/// the network, providing insight into network congestion and transaction fee trends.
/// * The metric is a counter, meaning it only increases over time.
/// * Fees are labeled by the block height and the token used for gas payments.

/// ### Example
/// ```
/// # HELP namada_fees_counter Total fees paid per block and per token
/// # TYPE namada_fees_counter counter
/// namada_fees_counter{height="777604",token="tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7",chain_id="housefire-alpaca.cc0d3e0c033be"} 0.5845009999999999
/// namada_fees_counter{height="777605",token="tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7",chain_id="housefire-alpaca.cc0d3e0c033be"} 0.154409
/// ```
use prometheus_exporter::prometheus::core::{AtomicF64, GenericCounterVec};
use prometheus_exporter::prometheus::{CounterVec, Opts, Registry};

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
        for tx in &block.transactions {
            let amount_per_gas = tx.fee.amount_per_gas_unit.parse::<f64>();
            let gas_limit = tx.fee.gas.parse::<f64>();

            let fee = match (amount_per_gas, gas_limit) {
                (Ok(amount_per_gas), Ok(gas_limit)) => amount_per_gas * gas_limit,
                _ => continue,
            };

            self.fees_counter
                .with_label_values(&[&block.height.to_string(), &tx.fee.gas_token])
                .inc_by(fee);
        }
    }
}

impl Default for Fees {
    fn default() -> Self {
        let fees_counter_opts =
            Opts::new("fees_counter", "Total fees paid per block and per token");
        let fees_counter = CounterVec::new(fees_counter_opts, &["height", "token"])
            .expect("unable to create int counter for transaction kinds");

        Self { fees_counter }
    }
}
