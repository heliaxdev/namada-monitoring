use std::collections::HashMap;

use namada_sdk::{ibc, token::Amount};

use crate::checks::AppConfig;

use super::CheckTrait;

const FLOW_IBC_CHECK_ID: &str = "transfer_limit_check";

#[derive(Debug, Clone)]
struct DefaultFeeThreshold {
    name: String,
    value: u64,
}

#[derive(Default)]
pub struct TransferLimitCheck {
    thresholds: HashMap<String, DefaultFeeThreshold>,
}

impl TransferLimitCheck {
    fn populate_thresholds(&mut self, config: &AppConfig) {
        for (address, threshold) in config.get_config().tokens_thresholds() {
            let token = address.clone();
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
impl CheckTrait for TransferLimitCheck {
    async fn check(&self, state: &crate::state::State) -> Vec<crate::shared::alert::Alert> {
        let last_state = state.last_block();
        let mut alerts = Vec::new();

        for wrapper in last_state.block.transactions {
            if !wrapper.is_successful() {
                continue;
            }

            for inner in wrapper.inners {
                if !inner.was_applied {
                    continue;
                }

                match inner.kind {
                    crate::shared::namada::InnerKind::Transfer(transfer) => {
                        let sources = transfer.sources;
                        for (account, amount) in sources {
                            let token = account.token.to_string();
                            let (token_name, token_threshold) =
                                if let Some(threshold) = self.thresholds.get(&token) {
                                    (threshold.name.clone(), Amount::from_u64(threshold.value))
                                } else {
                                    continue;
                                };

                            if !(amount.amount() > token_threshold) {
                                continue;
                            }

                            let alert = crate::shared::alert::Alert {
                                check_id: FLOW_IBC_CHECK_ID.to_string(),
                                severity: crate::shared::alert::Severity::Medium,
                                title: "Transfer exceeded threshold".to_string(),
                                description: format!(
                                    "Source {} made a transfer of {} {}, which exceeds the threshold of {}",
                                    &account.owner, amount.amount(), token_name, token_threshold
                                ),
                                metadata: crate::shared::alert::Metadata {
                                    block_height: Some(last_state.block.height as u32),
                                    tx_id: Some(inner.id.clone())
                                },
                                continous: self.is_continous(),
                                trigger_after: None,
                            };
                            alerts.push(alert);
                        }
                    }
                    crate::shared::namada::InnerKind::IbcMsgTransfer(ibc_message) => {
                        let sources = match ibc_message {
                            ibc::IbcMessage::Transfer(msg_transfer) => {
                                if let Some(transfer) = msg_transfer.transfer {
                                    transfer.sources
                                } else {
                                    continue;
                                }
                            }
                            _ => continue,
                        };

                        for (account, amount) in sources {
                            let token = account.token.to_string();
                            let (token_name, token_threshold) =
                                if let Some(threshold) = self.thresholds.get(&token) {
                                    (threshold.name.clone(), Amount::from_u64(threshold.value))
                                } else {
                                    continue;
                                };

                            if amount.amount().can_spend(&token_threshold) {
                                continue;
                            }

                            let alert = crate::shared::alert::Alert {
                                check_id: FLOW_IBC_CHECK_ID.to_string(),
                                severity: crate::shared::alert::Severity::Medium,
                                title: "IBC Transfer exceeded threshold".to_string(),
                                description: format!(
                                    "Source {} made an IBC transfer of {} {}, which exceeds the threshold of {}",
                                    &account.owner, amount.amount(), token_name, token_threshold
                                ),
                                metadata: crate::shared::alert::Metadata {
                                    block_height: Some(last_state.block.height as u32),
                                    tx_id: Some(inner.id.clone())
                                },
                                continous: self.is_continous(),
                                trigger_after: None,
                            };
                            alerts.push(alert);
                        }
                    }
                    _ => continue,
                }
            }
        }

        alerts
    }

    fn is_continous(&self) -> bool {
        false
    }
}
