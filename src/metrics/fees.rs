use crate::state::State;
/// ## Fees Metric. (fees)
/// This metric tracks the total transaction fees paid per token. It helps monitor the gas costs of transactions on
/// the network, providing insight into network congestion and transaction fee trends.
/// * The metric is a counter, meaning it only increases over time.
/// * Fees are labeled by the token used for gas payments.

/// ### Example
/// ```
/// # HELP namada_fees Total fees paid per token over time
/// # TYPE namada_fees counter
/// namada_fees{token="tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7",chain_id="housefire-alpaca.cc0d3e0c033be"} 0.5845009999999999
/// namada_fees{token="tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7",chain_id="housefire-alpaca.cc0d3e0c033be"} 0.154409
/// ```
use prometheus_exporter::prometheus::core::{AtomicF64, GenericCounterVec};
use prometheus_exporter::prometheus::{CounterVec, Opts, Registry};

use super::MetricTrait;

pub struct Fees {
    /// fees counter
    fees: GenericCounterVec<AtomicF64>,
}

impl MetricTrait for Fees {
    fn register(&self, registry: &Registry) -> anyhow::Result<()> {
        registry.register(Box::new(self.fees.clone()))?;
        Ok(())
    }

    fn reset(&self, _state: &State) {
        self.fees.reset();
    }

    fn update(&self, pre_state: &State, post_state: &State) {
        let block = post_state.get_last_block();
        for tx in &block.transactions {
            let amount_per_gas = tx.fee.amount_per_gas_unit.parse::<f64>();
            let gas_limit = tx.fee.gas.parse::<f64>();

            let fee = match (amount_per_gas, gas_limit) {
                (Ok(amount_per_gas), Ok(gas_limit)) => amount_per_gas * gas_limit,
                _ => continue,
            };

            self.fees
                .with_label_values(&[&tx.fee.gas_token])
                .inc_by(fee);
        }
    }
}

impl Default for Fees {
    fn default() -> Self {
        let fees_opts = Opts::new("fees", "Total fees paid per block and per token");
        let fees = CounterVec::new(fees_opts, &["token"])
            .expect("unable to create int counter for transaction kinds");

        Self { fees }
    }
}
