use std::str::FromStr;

use anyhow::Context;
use futures::FutureExt;
use namada_sdk::{hash::Hash, io::Client, rpc, state::Key};
use tendermint_rpc::{HttpClient, Url};

use crate::shared::{
    checksums::Checksums,
    namada::{Block, Epoch, Height},
};

pub struct Rpc {
    pub clients: Vec<HttpClient>,
}

impl Rpc {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            clients: urls
                .iter()
                .map(|url| {
                    let url = Url::from_str(url).unwrap();
                    HttpClient::new(url).unwrap()
                })
                .collect(),
        }
    }

    pub async fn query_tx_code_hash(&self, tx_code_path: &str) -> anyhow::Result<Option<String>> {
        let hash_key = Key::wasm_hash(tx_code_path);

        let futures = self
            .clients
            .iter()
            .map(|client| rpc::query_storage_value_bytes(client, &hash_key, None, false).boxed());

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        if let Some(tx_code_bytes) = res.context("Should be able to get tx code")?.0 {
            Ok(Hash::try_from(&tx_code_bytes[..])
                .ok()
                .map(|hash| hash.to_string()))
        } else {
            Ok(None)
        }
    }

    pub async fn query_current_epoch(&self, block_height: Height) -> anyhow::Result<Option<Epoch>> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::query_epoch_at_height(client, block_height.into()).boxed());

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        res.map(|epoch| epoch.map(|epoch| epoch.0))
            .context("Should be able to get epoch")
    }

    pub async fn query_block(
        &self,
        block_height: Height,
        checksums: &Checksums,
        epoch: Epoch,
    ) -> anyhow::Result<Block> {
        let futures = self.clients.iter().map(|client| client.block(block_height));

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        res.map(|response| Block::from(response, checksums, epoch))
            .context("Should be able to query for block")
    }
}
