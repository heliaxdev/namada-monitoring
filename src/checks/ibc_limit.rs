use super::CheckTrait;

const LIMIT_IBC_CHECK_ID: &str = "ibc_limit_check";

const LIMIT_THRESHOLD: f64 = 0.80;

#[derive(Default)]
pub struct IbcLimitCheck {}

#[async_trait::async_trait]
impl CheckTrait for IbcLimitCheck {
    async fn check(&self, state: &crate::state::State) -> Vec<crate::shared::alert::Alert> {
        let last_state = state.last_block();
        let mut alerts = Vec::new();

        for supply in &last_state.supplies {
            let limit = last_state.mint_limit.get(&supply.token);
            if let Some(limit) = limit {
                if *limit > 0 && supply.total as f64 > *limit as f64 * LIMIT_THRESHOLD {
                    let alert = crate::shared::alert::Alert {
                        check_id: format!("{}_{}", LIMIT_IBC_CHECK_ID, supply.token),
                        severity: crate::shared::alert::Severity::Low,
                        title: "IBC Token Supply Limit Alert".to_string(),
                        description: format!(
                            "IBC token {} supply {} almost reached limit {}",
                            supply.token, supply.total, limit
                        ),
                        metadata: crate::shared::alert::Metadata {
                            block_height: Some(last_state.block.height as u32),
                            tx_id: None,
                        },
                        continous: self.is_continous(),
                        trigger_after: Some(std::time::Duration::from_secs(60 * 60 * 6)), // Trigger after 6 hours
                    };
                    alerts.push(alert);
                }
            }
        }

        alerts
    }

    fn is_continous(&self) -> bool {
        false
    }
}
