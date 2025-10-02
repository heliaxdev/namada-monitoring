pub mod log;
pub mod slack;
pub mod telegram;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::Log;
use ttl_cache::TtlCache;

use crate::{
    checks::AppConfig,
    shared::{alert::Alert, block_explorer::BlockExplorer},
};

pub struct AlertManager {
    pub communication: Vec<Box<dyn AlertTrait>>,
    on_fire: TtlCache<String, (Alert, String)>,
}

impl AlertManager {
    pub fn new(app_config: &AppConfig) -> Self {
        let config = app_config.get_config();
        let any_alert_config = config.slack.is_some() || config.telegram.is_some();

        if !any_alert_config {
            return Self {
                communication: vec![Box::new(Log::new(config.block_explorer.clone()))],
                on_fire: TtlCache::new(100),
            };
        };

        let mut alerts: Vec<Box<dyn AlertTrait>> = vec![];

        if let Some(slack_config) = config.slack.clone() {
            let slack_alert = slack::SlackAlert::new(
                config.block_explorer.clone(),
                slack_config,
                app_config.chain_id.clone(),
            );
            alerts.push(Box::new(slack_alert));
        };

        if let Some(telegram_config) = config.telegram.clone() {
            let telegram_alert = telegram::TelegramAlert::new(
                config.block_explorer.clone(),
                telegram_config,
                app_config.chain_id.clone(),
            );
            alerts.push(Box::new(telegram_alert));
        };

        Self {
            communication: alerts,
            on_fire: TtlCache::new(100),
        }
    }

    pub async fn run_alerts(&mut self, alerts: Vec<Alert>) {
        for communication in &self.communication {
            for alert in &alerts {
                let id = format!("{}-{}", communication.get_id(), alert.check_id);
                let alert_trigger_after = alert.trigger_after;

                if alert_trigger_after.is_some() && self.on_fire.contains_key(&id) {
                    tracing::debug!(
                        "Alert {} already on fire, skipping sending again",
                        alert.check_id
                    );
                    continue;
                }

                if let Err(err) = communication.send_alerts(alert.clone()).await {
                    tracing::error!("Failed to send alert: {}", err);
                    continue;
                }

                let utc: DateTime<Utc> = Utc::now();
                self.on_fire.insert(
                    id,
                    (alert.clone(), utc.to_rfc3339()),
                    alert_trigger_after.unwrap_or_default(),
                );
            }
        }

        // send resolved text
        let mut to_remove = vec![];
        let mut tmp = self.on_fire.clone();
        for (id, (firing_alert, date)) in tmp.iter().filter(|(_, (alert, _))| alert.continous) {
            if alerts
                .iter()
                .any(|alert| firing_alert.check_id.eq(&alert.check_id))
            {
                continue;
            }

            for communication in &self.communication {
                if !id.starts_with(&communication.get_id()) {
                    continue;
                }

                if let Err(err) = communication.send_resolve(firing_alert.clone(), date).await {
                    tracing::error!("Failed to send alert: {}", err);
                    continue;
                }
            }

            to_remove.push(id);
        }

        for id in to_remove {
            self.on_fire.remove(id);
        }
    }
}

#[async_trait]
pub trait AlertTrait: Send + Sync {
    async fn send_alerts(&self, alert: Alert) -> Result<String, String>;
    async fn send_resolve(&self, alert: Alert, date: &str) -> Result<(), String>;
    fn get_block_explorer(&self) -> BlockExplorer;
    fn get_id(&self) -> String;
}
