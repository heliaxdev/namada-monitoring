use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    alerts::AlertManager,
    checks::CheckManager,
    metrics::MetricsExporter,
    rpc::Rpc,
    state::{BlockState, State},
};

pub struct Manager {
    pub metrics_exporter: MetricsExporter,
    pub checks: CheckManager,
    pub alerts: AlertManager,
    pub rpc: Rpc,
    pub state: State,
}

impl Manager {
    pub async fn new(config: &crate::config::AppConfig) -> (Arc<RwLock<Self>>, u32) {
        let metrics_exporter = MetricsExporter::new(config);
        let checks = CheckManager::new(config);
        let alerts = AlertManager::new(config);
        let rpc = Rpc::new(&config.rpc)
            .await
            .expect("Should be able to create RPC client");

        let initial_block_height = match config.initial_block_height {
            u32::MAX => rpc
                .query_lastest_height()
                .await
                .expect("Should be able to query latest block height"),
            height => height,
        };

        metrics_exporter
            .start_exporter()
            .expect("Should be able to start metrics exporter");

        let manager = Self {
            metrics_exporter,
            checks,
            alerts,
            rpc,
            state: State::default(),
        };

        for check in manager.checks.get_checks() {
            tracing::info!("Loaded check: {}", check);
        }

        (Arc::new(RwLock::new(manager)), initial_block_height)
    }

    pub fn has_enough_blocks(&self) -> bool {
        self.state.total_blocks() > 1
    }

    pub async fn update_next_state(
        &mut self,
        block_height: u64,
        tokens: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let last_epoch = if self.has_enough_blocks() {
            self.state.last_block().block.epoch
        } else {
            0
        };

        let checksums = self.rpc.query_checksums_at_height(block_height).await?;
        let epoch = self
            .rpc
            .query_epoch_at_height(block_height)
            .await?
            .expect("Epoch should be defined");
        let block = self
            .rpc
            .query_block_at_height(block_height, &checksums, epoch)
            .await?;

        let validators = if epoch.eq(&last_epoch) {
            self.state.last_block().validators.clone()
        } else {
            self.rpc.query_validators(epoch).await?
        };
        let (bonds, unbonds) = self.rpc.query_future_bonds_and_unbonds(epoch).await?;

        let mut supplies = vec![];
        let mut mint_limit = HashMap::new();
        for (alias, address) in tokens {
            let supply = if alias.contains("nam") {
                self.rpc.query_native_token_supply(&address).await?
            } else {
                self.rpc.query_token_supply(&address).await?
            };
            let limit = self.rpc.query_token_ibc_limit(&address).await?;
            mint_limit.insert(address.clone(), limit);
            supplies.push(supply);
        }

        let block_state = BlockState::new(block, bonds, unbonds, validators, supplies, mint_limit);
        self.state.add_block(block_state.clone());

        Ok(())
    }
}
