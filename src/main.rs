pub mod apprise;
pub mod checks;
pub mod config;
pub mod error;
pub mod log;
pub mod rpc;
pub mod shared;
pub mod state;

use std::{net::SocketAddr, sync::Arc, u64};

use anyhow::Context;
use apprise::AppRise;
use checks::all_checks;
use clap::Parser;
use config::AppConfig;
use error::AsRetryError;
use prometheus_exporter::prometheus::Registry;
use rpc::Rpc;
use shared::checksums::Checksums;
use state::State;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();
    config.log.init();

    let apprise = AppRise::new(config.apprise_url, config.slack_token, config.slack_channel);

    let retry_strategy = retry_strategy().max_delay_millis(config.sleep_for*1000);
    let rpc = Rpc::new(config.cometbft_urls);

    let initial_block_height = match config.initial_block_height {
        u64::MAX => rpc.query_lastest_height().await?,
        height => height,
    };

    let checksums = get_checksums_at_height(&rpc, initial_block_height).await?;
    let native_token = rpc.query_native_token().await.into_retry_error()?;

    let state = Arc::new(RwLock::new(State::new(
        checksums,
        initial_block_height,
    )));

    let registry = state.try_read()?.prometheus_registry().clone();
    start_prometheus_exporter(registry, config.prometheus_port)?;

    loop {
        Retry::spawn_notify(
            retry_strategy.clone(),
            || async {
                // lock is dropped at the end of the line
                let mut pre_state = state.read().await.clone();
                
                // immediate next block
                let block_height = pre_state.next_block_height();

                let epoch = rpc
                    .query_current_epoch(block_height)
                    .await
                    .into_retry_error()?
                    .unwrap_or(0);
                let block = rpc
                    .query_block(block_height, &pre_state.checksums, epoch)
                    .await
                    .into_retry_error()?;
                let total_supply_native = rpc
                    .query_total_supply(&native_token)
                    .await
                    .into_retry_error()?;
                let max_block_time_estimate = rpc
                    .query_max_block_time_estimate()
                    .await
                    .into_retry_error()?;

                let mut post_state_lock = state.write().await;
                post_state_lock.update(block, total_supply_native, max_block_time_estimate);
                let mut post_state = post_state_lock.clone();
                drop(post_state_lock);

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
                        }
                        checks::Checks::BlockTimeCheck(check) => {
                            check.run(&mut pre_state, &mut post_state).await
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

                Ok(())
            },
            notify,
        )
        .await?;
    }
}

fn start_prometheus_exporter(registry: Registry, port: u64) -> anyhow::Result<()> {
    let addr_raw = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr_raw.parse().context("can not parse listen addr")?;

    let mut builder = prometheus_exporter::Builder::new(addr);
    builder.with_registry(registry);
    builder.start().context("can not start exporter")?;

    Ok(())
}

fn retry_strategy() -> ExponentialBackoff {
    ExponentialBackoff::from_millis(1000)
        .factor(1)
        .max_delay_millis(10000)
}
