use std::{fmt::Display, time::Duration};

use crate::shared::alert::Metadata;

use super::{AppConfig, CheckTrait};

const BLOCK_CHECK_ID: &str = "block_check";

pub struct BlockCheck {
    estimated_block_time: f64,
    block_time_deviation: f64,
}

#[async_trait::async_trait]
impl CheckTrait for BlockCheck {
    async fn check(&self, state: &super::State) -> Vec<crate::shared::alert::Alert> {
        let last_state = state.last_block();
        let prev_state = state.prev_block();

        let process_time = last_state.block.timestamp - prev_state.block.timestamp;
        let deviation = self.estimated_block_time * self.block_time_deviation;

        if process_time as f64 > self.estimated_block_time + deviation {
            vec![crate::shared::alert::Alert {
                check_id: BLOCK_CHECK_ID.to_string(),
                title: "High block time".to_string(),
                description: format!(
                    "Block time for block {} was *{}* seconds",
                    last_state.block.height, process_time
                ),
                severity: crate::shared::alert::Severity::Medium,
                metadata: Metadata::new(Some(last_state.block.height as u32), None),
                trigger_after: Some(Duration::from_secs(60)),
                continous: self.is_continous(),
            }]
        } else {
            vec![]
        }
    }

    fn is_continous(&self) -> bool {
        false
    }
}

impl Display for BlockCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlockCheck(estimated_block_time: {}, block_time_deviation: {})",
            self.estimated_block_time, self.block_time_deviation
        )
    }
}

impl BlockCheck {
    pub fn new(config: &AppConfig) -> Self {
        let config = config.get_config();
        Self {
            estimated_block_time: config.chain.block_time,
            block_time_deviation: config.chain.block_time_max_deviation,
        }
    }
}
