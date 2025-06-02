use crate::{
    constants::MICRO,
    shared::alert::{Alert, Metadata, Severity},
    state::State,
};

use super::{AppConfig, CheckTrait};
use std::collections::HashMap;

const FEE_CHECK_ID: &str = "fee_check";

#[derive(Debug, Clone)]
struct DefaultFeeThreshold {
    name: String,
    value: f64,
}

#[derive(Debug, Clone, Default)]
pub struct FeeCheck {
    thresholds: HashMap<String, DefaultFeeThreshold>,
}

impl FeeCheck {
    fn populate_thresholds(&mut self, config: &AppConfig) {
        for fee in config.get_config().fees.iter() {
            let token = fee.token.clone();
            let threshold = fee.threshold * MICRO;
            self.thresholds.insert(
                token.clone(),
                DefaultFeeThreshold {
                    name: token,
                    value: threshold,
                },
            );
        }
    }

    pub fn new(config: &AppConfig) -> Self {
        let mut instance = Self {
            thresholds: HashMap::new(),
        };
        instance.populate_thresholds(config);
        instance
    }
}

#[async_trait::async_trait]
impl CheckTrait for FeeCheck {
    async fn check(&self, state: &State) -> Vec<Alert> {
        let last_state = state.last_block();

        last_state.block.transactions.iter().filter_map(|tx| {
            let tx_id = tx.id.clone();
            let fee_token = tx.fee.gas_token.clone();
            let fee = tx.compute_fee();

            match self.thresholds.get(&fee_token) {
                Some(threshold) => {
                    if threshold.value >= fee {
                        None
                    } else {
                        let title = "Fee too high".to_string();
                        let description = format!("The transaction *{}* paid a fee of *{}* _{}_, which is above the threshold of *{}* _{}_.", tx_id, fee, threshold.name, threshold.value, threshold.name);
                        Some(Alert {
                            check_id: FEE_CHECK_ID.to_string(),
                            title,
                            description,
                            metadata: Metadata::new(
                                Some(last_state.block.block.header.height.value() as u32),
                                Some(tx_id.clone()),
                            ),
                            severity: Severity::Low,
                            trigger_after: None,
                        continous: self.is_continous()
                        })
                    }
                },
                None => None,
            }
        }).collect::<Vec<_>>()
    }

    fn is_continous(&self) -> bool {
        false
    }
}
