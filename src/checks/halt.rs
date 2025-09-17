use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::shared::alert::{Alert, Metadata, Severity};

use super::{AppConfig, CheckTrait};

const AVG_BLOCK_TIME_CHECK_ID: &str = "average_block_time_check";
const HALT_CHECK_ID: &str = "halt_check";

const AVG_TIME_THRESHOLD: f64 = 1.2;
const MIN_LOOKBACK_WINDOW: usize = 2;
const MAX_LOOKBACK_WINDOW: usize = 50;

pub struct HaltCheck {
    estimated_block_time: f64,
    halt_threshold: u64,
}

#[async_trait::async_trait]
impl CheckTrait for HaltCheck {
    async fn check(&self, state: &super::State) -> Vec<Alert> {
        let mut alerts = Vec::new();

        let blocks = &state.blocks;
        let num_blocks = blocks.len();

        if let Some(last_block) = blocks.last() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let last_block_time = last_block.block.timestamp;

            if now > last_block_time && (now - last_block_time) > self.halt_threshold as i64 {
                alerts.push(Alert {
                    check_id: HALT_CHECK_ID.to_string(),
                    title: "Chain might be halted".to_string(),
                    description: format!(
                        "No new block has been produced for more than {} seconds.",
                        self.halt_threshold
                    ),
                    severity: Severity::Critical,
                    metadata: Metadata::new(Some(last_block.block.height as u32), None),
                    trigger_after: Some(Duration::from_secs(60)),
                    continous: self.is_continous(),
                });
            }
        }

        if num_blocks >= MIN_LOOKBACK_WINDOW {
            let window_size = num_blocks.min(MAX_LOOKBACK_WINDOW);
            let start = num_blocks - window_size;
            let mut total_diffs = 0f64;
            let mut count = 0u32;

            for window in blocks[start..].windows(2) {
                let t1 = window[0].block.timestamp;
                let t2 = window[1].block.timestamp;
                if t2 > t1 {
                    total_diffs += (t2 - t1) as f64;
                    count += 1;
                }
            }

            if count > 0 {
                let avg = total_diffs / count as f64;
                if avg > self.estimated_block_time * AVG_TIME_THRESHOLD {
                    alerts.push(Alert {
                        check_id: AVG_BLOCK_TIME_CHECK_ID.to_string(),
                        title: "High average block time".to_string(),
                        description: format!(
                            "Average block time over the last {} blocks is *{}* seconds, which is higher than expected.",
                            window_size,
                            avg
                        ),
                        severity: Severity::Medium,
                        metadata: Metadata::new(
                            Some(blocks.last().unwrap().block.height as u32),
                            None,
                        ),
                        trigger_after: Some(Duration::from_secs(60)),
                        continous: self.is_continous(),
                    });
                }
            }
        }

        alerts
    }

    fn is_continous(&self) -> bool {
        true
    }
}

impl HaltCheck {
    pub fn new(config: &AppConfig) -> Self {
        let config = config.get_config();
        Self {
            estimated_block_time: config.chain.block_time,
            halt_threshold: config.chain.halt_threshold,
        }
    }
}
