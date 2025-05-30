use super::{AppConfig, CheckTrait};

const TX_SECTIONS_CHECK_ID: &str = "tx_sections_check";
const TX_BATCH_CHECK_ID: &str = "tx_batch_check";

pub struct TxCheck {
    pub threshold_sections: u64,
    pub threshold_batch: u64,
}

impl TxCheck {
    pub fn new(config: &AppConfig) -> Self {
        let config = config.get_config();
        Self {
            threshold_sections: config.tx.threshold_sections,
            threshold_batch: config.tx.threshold_batch,
        }
    }
}

#[async_trait::async_trait]
impl CheckTrait for TxCheck {
    async fn check(&self, states: &super::State) -> Vec<crate::shared::alert::Alert> {
        let last_block = states.last_block();

        let mut alerts = Vec::new();
        for transaction in &last_block.block.transactions {
            if transaction.total_sections
                > self.threshold_sections * transaction.inners.len() as u64
            {
                alerts.push(crate::shared::alert::Alert {
                    check_id: TX_SECTIONS_CHECK_ID.to_string(),
                    title: "Transaction sections limit exceeded".to_string(),
                    description: format!(
                        "Transaction {} has {} sections",
                        transaction.id, transaction.total_sections,
                    ),
                    metadata: crate::shared::alert::Metadata {
                        block_height: Some(last_block.block.height as u32),
                        tx_id: Some(transaction.id.clone()),
                    },
                    severity: crate::shared::alert::Severity::Low,
                    trigger_after: None,
                    continous: self.is_continous(),
                });
            }

            if transaction.inners.len() as u64 > self.threshold_batch {
                alerts.push(crate::shared::alert::Alert {
                    check_id: TX_BATCH_CHECK_ID.to_string(),
                    title: "Transaction batch limit exceeded".to_string(),
                    description: format!(
                        "Transaction {} has {} inner transactions",
                        transaction.id,
                        transaction.inners.len(),
                    ),
                    metadata: crate::shared::alert::Metadata {
                        block_height: Some(last_block.block.height as u32),
                        tx_id: Some(transaction.id.clone()),
                    },
                    severity: crate::shared::alert::Severity::Low,
                    trigger_after: None,
                    continous: self.is_continous(),
                });
            }
        }

        alerts
    }

    fn is_continous(&self) -> bool {
        false
    }
}
