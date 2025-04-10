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
use rpc::Rpc;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_retry2::{strategy::ExponentialBackoff, Retry};

fn notify(err: &std::io::Error, duration: std::time::Duration) {
    if duration.as_secs() > 100 {
        tracing::warn!("Error {err:?} occurred at {duration:?}");
    }
}

use slack_hook::{PayloadBuilder, Slack, SlackLink, SlackTextContent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = AppConfig::parse();
    config.log.init();

    let retry_strategy = retry_strategy(config.sleep_for);
    let mut rpc = Rpc::new(&config);

    let initial_block_height = match config.initial_block_height {
        u64::MAX => rpc.query_lastest_height().await?,
        height => height,
    };
    let last_block_height = config.last_block_height;

    // Check the given chain id matchs the one reported by the rpc
    let chain_id = rpc.get_chain_id().await?;
    match config.chain_id.as_ref() {
        Some(expected_chain_id) if &chain_id != expected_chain_id => {
            return Err(anyhow::anyhow!(
                "Chain ID mismatch: expected {}, got {}",
                expected_chain_id,
                chain_id
            ));
        }
        _ => config.chain_id = Some(chain_id),
    };

    let metrics = MetricsExporter::default_metrics(&config);
    let checks = CheckExporter::new(&config);

    let state = rpc.get_state(initial_block_height).await?;
    metrics.start_exporter_with(&state)?;

    let rpc = Arc::new(Mutex::new(Rpc::new(&config)));
    let current_state = Arc::new(RwLock::new(state));
    let block_explorer = config
        .block_explorer
        .unwrap_or("https://explorer75.org/namada/blocks/".to_string());

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
                        let text = [
                            SlackTextContent::Text(":rotating_light: Alert at height:".into()),
                            SlackTextContent::Link(SlackLink::new(
                                format!("{}/{}", block_explorer, block_height).as_str(),
                                format!("{}", block_height).as_str(),
                            )),
                            SlackTextContent::Text("\n".into()),
                        ];
                        let payload = PayloadBuilder::new()
                            .text(text.as_slice())
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
    ExponentialBackoff::from_millis(1)
        .factor(1)
        .max_delay_millis(max_delay_milis)
}
