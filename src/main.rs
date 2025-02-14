pub mod config;
pub mod error;
pub mod log;
pub mod metrics;
pub mod rpc;
pub mod shared;
pub mod state;

use clap::Parser;
use config::AppConfig;
use error::AsRetryError;
use metrics::MetricsExporter;
use rpc::Rpc;
use shared::checksums::Checksums;
use state::State;
use std::sync::Arc;
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
    let checksums = get_checksums_at_height(rpc, height).await?;
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
    let (future_bonds, future_unbonds) = rpc
        .query_future_bonds_and_unbonds(epoch)
        .await
        .into_retry_error()?;
    let validators = rpc.query_validators(epoch).await.into_retry_error()?;
    let peers = rpc.query_peers().await.into_retry_error()?;
    Ok(State::new(
        block,
        native_token,
        max_block_time_estimate,
        total_supply_native,
        validators,
        future_bonds,
        future_unbonds,
        peers,
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = AppConfig::parse();
    config.log.init();

    let retry_strategy = retry_strategy(config.sleep_for);
    let rpc = Rpc::new(config.rpc.clone());
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
    }

    let initial_block_height = match config.initial_block_height {
        u64::MAX => rpc.query_lastest_height().await?,
        height => height,
    };

    let metrics = MetricsExporter::default_metrics(&config);
    let state = get_state_from_rpc(&rpc, initial_block_height).await?;
    // metrics.reset(&state);
    metrics.start_exporter()?;

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

                // update metrics
                metrics.update(&pre_state, &post_state);

                // post_state is the new current state
                *current_state.write().await = post_state;
                tracing::info!("Done block {}", block_height);

                Ok(())
            },
            notify,
        )
        .await?;
    }
}

fn retry_strategy(max_delay_milis: u64) -> ExponentialBackoff {
    ExponentialBackoff::from_millis(1000)
        .factor(1)
        .max_delay_millis(max_delay_milis)
}
