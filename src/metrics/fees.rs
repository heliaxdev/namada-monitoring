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
use prometheus_exporter::prometheus::{GaugeVec, Registry, Opts};

use super::MetricTrait;

pub struct Fees {
    /// fees counters
    fees: GaugeVec,
}

impl MetricTrait for Fees {
    fn register(&self, registry: &Registry) -> anyhow::Result<()> {
        registry.register(Box::new(self.fees.clone()))?;
        Ok(())
    }

    fn reset(&self, state: &State) {
        let block = state.get_last_block();
        for tx in &block.transactions {
            let amount_per_gas = tx.fee.amount_per_gas_unit.parse::<f64>();
            let gas_limit = tx.fee.gas.parse::<f64>();

            let fee = match (amount_per_gas, gas_limit) {
                (Ok(amount_per_gas), Ok(gas_limit)) => amount_per_gas * gas_limit,
                _ => continue,
            };

            self.fees
                .with_label_values(&[&tx.fee.gas_token])
                .set(fee);
        }
    }

    fn update(&self, _pre_state: &State, post_state: &State) {
        self.reset(post_state); 
    }
}

impl Default for Fees {
    fn default() -> Self {
        let fees_opts = Opts::new("fees", "Total fees paid per token over time");
        let fees = GaugeVec::new(fees_opts, &["token"])
            .expect("unable to create gauge vector for transaction fees");
        Self { fees }
    }
}
