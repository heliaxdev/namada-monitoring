pub mod apprise;
pub mod config;
pub mod log;
pub mod rpc;
pub mod shared;
pub mod state;

use std::{net::SocketAddr, thread::sleep, time::Duration};

use anyhow::Context;
use clap::Parser;
use config::AppConfig;
use prometheus_exporter::prometheus::Registry;
use rpc::Rpc;
use shared::checksums::Checksums;
use state::State;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();
    config.log.init();

    let rpc = Rpc::new(config.cometbft_urls);

    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = rpc
            .query_tx_code_hash(&code_path)
            .await?
            .unwrap_or_else(|| panic!("{} must be defined in namada storage.", code_path));
        checksums.add(code_path, code);
    }

    let mut state = State::new(checksums);
    let registry = state.prometheus_registry();

    start_prometheus_exporter(registry, config.prometheus_port)?;

    loop {
        let block_height = state.next_block_height();
        let epoch = rpc.query_current_epoch(block_height).await?.unwrap_or(0);
        let block = rpc
            .query_block(block_height, &state.checksums, epoch)
            .await?;

        state.update(block);

        tracing::info!("Done block {}", block_height);
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
