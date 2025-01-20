use std::str::FromStr;

use anyhow::Context;
use futures::FutureExt;
use namada_sdk::{
    address::Address as NamadaAddress, hash::Hash, io::Client, rpc, state::Epoch as NamadaEpoch,
    state::Key,
};
use tendermint_rpc::{HttpClient, Url};

use crate::shared::{
    checksums::Checksums,
    namada::{Address, Block, BlockResult, Epoch, Height, Validator},
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

        let events_futures = self
            .clients
            .iter()
            .map(|client| client.block_results(block_height));
        let (events_res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(events_futures).await;

        let events = events_res
            .map(|response| BlockResult::from(response))
            .context("Should be able to query for block events")?;

        res.map(|response| Block::from(response, events, checksums, epoch))
            .context("Should be able to query for block")
    }

    pub async fn query_validators(&self, epoch: Epoch) -> anyhow::Result<Vec<Validator>> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::get_all_consensus_validators(client, NamadaEpoch(epoch)).boxed());

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        res.context("Should be able to query native token")
            .map(|set| {
                set.into_iter()
                    .map(|validator| Validator {
                        address: validator.address.to_string(),
                        voting_power: validator.bonded_stake.raw_amount().as_u64(),
                    })
                    .collect()
            })
    }

    pub async fn query_native_token(&self) -> anyhow::Result<Address> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::query_native_token(client).boxed());

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        res.context("Should be able to query native token")
            .map(|address| address.to_string())
    }

    pub async fn query_total_supply(&self, native_token: &str) -> anyhow::Result<u64> {
        let address = NamadaAddress::from_str(native_token)
            .context("Should be able to convert string to address")?;

        let futures = self
            .clients
            .iter()
            .map(|client| rpc::get_token_total_supply(client, &address).boxed());

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        res.context("Should be able to query native token")
            .map(|amount| amount.raw_amount().as_u64())
    }

    pub async fn query_future_bonds_and_unbonds(&self, epoch: Epoch) -> anyhow::Result<(u64, u64)> {
        let pipeline_epoch = NamadaEpoch(epoch + 1);
        let futures = self.clients.iter().map(|client| {
            rpc::enriched_bonds_and_unbonds(client, pipeline_epoch, &None, &None).boxed()
        });

        let (res, _ready_future_index, _remaining_futures) =
            futures::future::select_all(futures).await;

        res.context("Should be able to query native token")
            .map(|summary| {
                (
                    summary
                        .bonds_total_active()
                        .map(|amount| amount.raw_amount().as_u64())
                        .unwrap_or(0),
                    summary
                        .unbonds_total_active()
                        .map(|amount| amount.raw_amount().as_u64())
                        .unwrap_or(0),
                )
            })
    }
}
