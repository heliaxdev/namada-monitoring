use std::fmt::Display;

use crate::shared::alert::Metadata;

use super::{AppConfig, CheckTrait};

const GAS_CHECK_ID: &str = "gas_check";

pub struct GasCheck {
    gas_limit_threshold: f64,
}

#[async_trait::async_trait]
impl CheckTrait for GasCheck {
    async fn check(&self, state: &super::State) -> Vec<crate::shared::alert::Alert> {
        let last_state = state.last_block();

        last_state
            .block
            .transactions
            .iter()
            .filter_map(|transaction| {
                let gas_limit = transaction.fee.gas.parse::<f64>().unwrap_or_default();
                let gas_used = transaction.fee.gas_used as f64;

                if gas_used * (1.0 + self.gas_limit_threshold) < gas_limit {
                    Some(crate::shared::alert::Alert {
                        check_id: GAS_CHECK_ID.to_string(),
                        title: "Gas limit too high".to_string(),
                        description: format!(
                            "Gas limit is for tx *{}* too high: gas limit is *{}*, gas used is *{}*",
                            transaction.id, gas_limit, gas_used
                        ),
                        metadata: Metadata {
                            block_height: Some(last_state.block.height as u32),
                            tx_id: Some(transaction.id.clone()),
                        },
                        severity: crate::shared::alert::Severity::Low,
                        trigger_after: None,
                        continous: self.is_continous(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_continous(&self) -> bool {
        false
    }
}

impl Display for GasCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GasCheck(gas_limit_threshold: {})",
            self.gas_limit_threshold
        )
    }
}

impl GasCheck {
    pub fn new(config: &AppConfig) -> Self {
        let config = config.get_config();
        Self {
            gas_limit_threshold: config.tx.gas_limit_threshold,
        }
    }
}
