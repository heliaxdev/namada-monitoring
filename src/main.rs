pub mod apprise;
pub mod checks;
pub mod config;
pub mod error;
pub mod log;
pub mod rpc;
pub mod shared;
pub mod state;

use std::{sync::Arc, u64};

use apprise::AppRise;
use checks::all_checks;
use clap::Parser;
use config::AppConfig;
use error::AsRetryError;
use rpc::Rpc;
use shared::checksums::Checksums;
use state::{PrometheusMetrics, State};
use tokio::sync::RwLock;
use tokio_retry2::{strategy::ExponentialBackoff, Retry};

fn notify(err: &std::io::Error, duration: std::time::Duration) {
    if duration.as_secs() > 100 {
        tracing::warn!("Error {err:?} occurred at {duration:?}");
    }
}

async fn get_checksums_at_height(rpc: &Rpc, height: u64) -> anyhow::Result<Checksums> {
    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = rpc
            .query_tx_code_hash(&code_path, height)
            .await?
            .unwrap_or_else(|| panic!("{} must be defined in namada storage.", code_path));
        checksums.add(code_path, code);
    }
    Ok(checksums)
}

async fn get_state_from_rpc(rpc: &Rpc, height: u64) -> anyhow::Result<State> {
    let checksums = get_checksums_at_height(&rpc, height).await?;
    let native_token = rpc.query_native_token().await.into_retry_error()?;

    let epoch = rpc
        .query_current_epoch(height)
        .await
        .into_retry_error()?
        .unwrap_or(0);
    let block = rpc
        .query_block(height, &checksums, epoch)
        .await
        .into_retry_error()?;
    let max_block_time_estimate = rpc
        .query_max_block_time_estimate()
        .await
        .into_retry_error()?;
    let total_supply_native = rpc
        .query_total_supply(&native_token)
        .await
        .into_retry_error()?;

    Ok(State::new(
        block,
        checksums,
        native_token,
        max_block_time_estimate,
        total_supply_native,
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();
    config.log.init();

    let apprise = AppRise::new(config.apprise_url, config.slack_token, config.slack_channel);

    let retry_strategy = retry_strategy().max_delay_millis(config.sleep_for * 1000);
    let rpc = Rpc::new(config.cometbft_urls);

    let initial_block_height = match config.initial_block_height {
        u64::MAX => rpc.query_lastest_height().await?,
        height => height,
    };

    let state = get_state_from_rpc(&rpc, initial_block_height).await?;
    let metrics = PrometheusMetrics::new();
    metrics.reset_metrics(&state);
    metrics.start_exporter(config.prometheus_port)?;

    let current_state = Arc::new(RwLock::new(state));
    loop {
        Retry::spawn_notify(
            retry_strategy.clone(),
            || async {
                // lock is dropped at the end of the line
                let pre_state = current_state.read().await.clone();

                // immediate next block
                let block_height = pre_state.next_block_height();
                let post_state = get_state_from_rpc(&rpc, block_height)
                    .await
                    .into_retry_error()?;

                for check_kind in all_checks() {
                    let check_res = match check_kind {
                        checks::Checks::BlockHeightCheck(check) => {
                            check.run(&pre_state, &post_state).await
                        }
                        checks::Checks::EpochCheck(check) => {
                            check.run(&pre_state, &post_state).await
                        }
                        checks::Checks::TotalSupplyNative(check) => {
                            check.run(&pre_state, &post_state).await
                        },
                        checks::Checks::TxSize(check) => {
                            check.run(&pre_state, &post_state).await
                        }
                        checks::Checks::BlockTimeCheck(check) => {
                            check.run(&pre_state, &post_state).await
                        }
                    };
                    if let Err(error) = check_res {
                        tracing::error!("Error: {}", error.to_string());
                        apprise
                            .send_to_slack(error.to_string())
                            .await
                            .into_retry_error()?;
                    }
                }

                // update metrics
                metrics.update(&pre_state, &post_state);

                // post_state is the new current state
                *current_state.write().await = post_state;
                Ok(())
            },
            notify,
        )
        .await?;
    }
}

fn retry_strategy() -> ExponentialBackoff {
    ExponentialBackoff::from_millis(1000)
        .factor(1)
        .max_delay_millis(10000)
}
