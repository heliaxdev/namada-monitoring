pub mod checks;
pub mod config;
pub mod error;
pub mod log;
pub mod metrics;
pub mod rpc;
pub mod shared;
pub mod state;

use checks::CheckExporter;
use clap::Parser;
use config::AppConfig;
use error::AsRetryError;
use metrics::MetricsExporter;
use regex::Regex;
use rpc::Rpc;
use slack_hook::{PayloadBuilder, Slack, SlackLink, SlackText, SlackTextContent};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_retry2::{strategy::ExponentialBackoff, Retry};

fn notify(err: &std::io::Error, duration: std::time::Duration) {
    if duration.as_secs() > 100 {
        tracing::warn!("Error {err:?} occurred at {duration:?}");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();
    config.log.init();

    let retry_strategy = retry_strategy(config.sleep_for);
    let rpc = Arc::new(Mutex::new(Rpc::new(&config.rpc, &config.chain_id).await));

    let initial_block_height = match config.initial_block_height {
        u64::MAX => rpc.lock().await.query_lastest_height().await?,
        height => height,
    };
    let last_block_height = config.last_block_height;

    let metrics = MetricsExporter::default_metrics(&config);
    let checks = CheckExporter::new(&config);

    // retry 10 times to get the first state
    let state = Retry::spawn_notify(
        retry_strategy.clone(),
        || async {
            let mut rpc = rpc.lock().await;
            let state = rpc
                .get_state(initial_block_height)
                .await
                .into_retry_error()?;
            Ok(state)
        },
        notify,
    )
    .await?;
    metrics.start_exporter_with(&state)?;

    let current_state = Arc::new(RwLock::new(state));
    let block_explorer = config
        .block_explorer
        .unwrap_or("https://explorer75.org/namada".to_string());
    let re = Regex::new(r"<([^< ]+)\|([^> ]+)>").unwrap();

    loop {
        if Retry::spawn_notify(
            retry_strategy.clone(),
            || async {
                // lock is dropped at the end of the line
                let pre_state = current_state.read().await.clone();

                // wait for immediate next block
                let block_height = pre_state.next_block_height();
                let post_state = rpc
                    .lock()
                    .await
                    .get_state(block_height)
                    .await
                    .into_retry_error()?;

                if block_height <= last_block_height {
                    // update metrics
                    metrics.update(&pre_state, &post_state);

                    let alerts = checks.run_checks(&[&pre_state, &post_state]);
                    if !alerts.is_empty() {
                        let mut alert_content = vec![];
                        alert_content.push(SlackTextContent::Text(SlackText::new(format!(
                            ":bricks: {} - Alerts at height:",
                            config.chain_id.clone()
                        ))));
                        alert_content.push(SlackTextContent::Link(SlackLink::new(
                            &format!("{}/blocks/{}", block_explorer, block_height),
                            &block_height.to_string(),
                        )));
                        alert_content.push(SlackTextContent::Text(SlackText::new("\n")));

                        for alert in alerts {
                            let mut last_end = 0;
                            for cap in re.captures_iter(&alert) {
                                let m = cap.get(0).unwrap();
                                // Add text between matches
                                if m.start() > last_end {
                                    alert_content.push(SlackTextContent::Text(SlackText::new(
                                        &alert[last_end..m.start()],
                                    )));
                                }
                                // Add capture groups
                                let url = cap.get(1).map_or("", |m| m.as_str());
                                let text = cap.get(2).map_or("", |m| m.as_str());
                                alert_content
                                    .push(SlackTextContent::Link(SlackLink::new(url, text)));
                                last_end = m.end();
                            }

                            // Add remaining text after last match
                            if last_end < alert.len() {
                                alert_content.push(SlackTextContent::Text(SlackText::new(
                                    &alert[last_end..],
                                )));
                            }
                        }
                        let payload = PayloadBuilder::new()
                            .text(alert_content.as_slice())
                            .build()
                            .expect("we know this payload is valid");
                        if config.slack.is_some() {
                            // send alerts to slack
                            let slack = Slack::new(config.slack.as_ref().unwrap()).unwrap();
                            tracing::info!("Sending message to slack {:?}", payload);
                            match slack.send(&payload).await {
                                Ok(()) => println!("Message sent!"),
                                Err(err) => eprintln!("Error: {err:?}"),
                            };
                        } else {
                            tracing::info!("ALERT! {:?}", payload);
                        }
                    }
                    // post_state is the new current state
                    *current_state.write().await = post_state;
                    tracing::info!("Done block {}", block_height);
                    Ok(false) // continue
                } else {
                    tracing::info!("Last block height reached: {}", block_height);
                    Ok(true) // done
                }
            },
            notify,
        )
        .await?
        {
            break Ok(());
        }
    }
}

fn retry_strategy(max_delay_milis: u64) -> ExponentialBackoff {
    ExponentialBackoff::from_millis(100)
        .factor(1)
        .max_delay_millis(max_delay_milis)
}
