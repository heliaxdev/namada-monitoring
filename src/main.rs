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
use rpc::Rpc;
use shared::checksums::Checksums;
use state::State;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();
    config.log.init();

    start_prometheus_exporter(config.prometheus_port)?;

    let rpc = Rpc::new(config.cometbft_urls);

    let mut checksums = Checksums::default();
    for code_path in Checksums::code_paths() {
        let code = rpc
            .query_tx_code_hash(&code_path)
            .await?
            .unwrap_or_else(|| panic!("{} must be defined in namada storage.", code_path));
        checksums.add(code_path, code);
    }

    let state = State::new(checksums);

    loop {
        println!("Test...");
        sleep(Duration::from_secs(3));
    }
}

fn start_prometheus_exporter(port: u64) -> anyhow::Result<()> {
    let addr_raw = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr_raw.parse().context("can not parse listen addr")?;
    prometheus_exporter::start(addr).context("can not start exporter")?;

    Ok(())
}
