pub mod alerts;
pub mod checks;
pub mod config;
pub mod constants;
pub mod error;
pub mod log;
pub mod metrics;
pub mod rpc;
pub mod shared;
pub mod state;

use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use async_stream::stream;
use clap::Parser;
use config::AppConfig;
use error::AsRetryError;
use futures::{pin_mut, Stream, StreamExt};
use shared::manager::Manager;
use tokio::signal;
use tokio_retry2::{strategy::FixedInterval, RetryIf};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();
    config.log.init();

    rlimit::increase_nofile_limit(10240).unwrap();
    rlimit::increase_nofile_limit(u64::MAX).unwrap();

    let tokens = config.get_config().tokens();

    tracing::info!("{:#?}", config.get_config());

    let (manager, initial_block_height) = Manager::new(&config).await;

    let retry_strategy = retry_strategy(config.sleep_for);
    let must_exit_handle = must_exit_handle();

    let s = indexes(initial_block_height, None);
    pin_mut!(s);

    while let Some(index) = s.next().await {
        if must_exit_handle.load(atomic::Ordering::Relaxed) {
            break;
        }
        _ = RetryIf::spawn(
            retry_strategy.clone(),
            || async {
                tracing::info!("Fetching block at height {}...", index);
                let mut manager = manager.write().await;

                let continous_alerts = manager.checks.run_continous_checks(&manager.state).await;

                manager
                    .update_next_state(index as u64, tokens.clone())
                    .await
                    .into_retry_error()?;

                if !manager.has_enough_blocks() {
                    return Ok(());
                }

                manager.metrics_exporter.update(&manager.state);

                let block_alerts = manager.checks.run_block_checks(&manager.state).await;

                let all_alerts = continous_alerts
                    .into_iter()
                    .chain(block_alerts.into_iter())
                    .collect::<Vec<_>>();
                manager.alerts.run_alerts(all_alerts.clone()).await;

                tracing::info!(
                    "Done block at height {} ({} alerts)",
                    index,
                    all_alerts.len()
                );

                Ok(())
            },
            |_e: &std::io::Error| !must_exit_handle.load(atomic::Ordering::Relaxed),
            notify,
        )
        .await;
    }

    Ok(())
}

fn notify(err: &std::io::Error, duration: std::time::Duration) {
    tracing::info!("Error {err:?} occurred at {duration:?}");
}

fn retry_strategy(sleep_for: u64) -> FixedInterval {
    FixedInterval::from_millis(sleep_for * 1000)
}

fn must_exit_handle() -> Arc<AtomicBool> {
    let handle = Arc::new(AtomicBool::new(false));
    let task_handle = Arc::clone(&handle);
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
        task_handle.store(true, atomic::Ordering::Relaxed);
    });
    handle
}

fn indexes(from: u32, to: Option<u32>) -> impl Stream<Item = u32> {
    stream! {
        for i in from..to.unwrap_or(u32::MAX) {
            yield i;
        }
    }
}
