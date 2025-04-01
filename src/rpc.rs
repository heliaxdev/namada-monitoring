use anyhow::Context;
use futures::FutureExt;
use namada_sdk::{
    address::Address as NamadaAddress,
    hash::Hash,
    proof_of_stake::types::ValidatorState,
    rpc,
    state::{BlockHeight, Epoch as NamadaEpoch, Key},
    tendermint::node::Id,
    tendermint_rpc::Client,
};
use std::{future::Future, str::FromStr};
use tendermint_rpc::client::CompatMode;
use tendermint_rpc::{endpoint::net_info::PeerInfo, HttpClient, HttpClientUrl, Url};

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
                    HttpClient::builder(HttpClientUrl::try_from(url).unwrap())
                        .compat_mode(CompatMode::V0_37)
                        .build()
                        .unwrap()
                })
                .collect(),
        }
    }

    pub async fn get_abci_info(&self) -> anyhow::Result<()> {
        for client in &self.clients {
            let abci_info = client.abci_info().await?;
            println!("abci_info: {:?}", abci_info);
        }
        Ok(())
    }

    pub async fn get_chain_id(&self) -> anyhow::Result<String> {
        let mut chain_id = None;
        for client in &self.clients {
            let current_chain_id = match client.status().await {
                Ok(status) => {
                    let network = status.node_info.network.clone();
                    String::from(network)
                }
                Err(err) => return Err(anyhow::anyhow!("Failed to get status: {:?}", err)),
            };

            if let Some(existing_chain_id) = &chain_id {
                if existing_chain_id != &current_chain_id {
                    return Err(anyhow::anyhow!(
                        "Chain IDs do not match: {} != {}",
                        existing_chain_id,
                        current_chain_id
                    ));
                }
            } else {
                chain_id = Some(current_chain_id);
            }
        }
        match chain_id {
            Some(chain_id) => Ok(chain_id.to_string()),
            None => Err(anyhow::anyhow!("No chain IDs found")),
        }
    }

    pub async fn query_tx_code_hash(
        &self,
        tx_code_path: &str,
        height: Height,
    ) -> anyhow::Result<Option<String>> {
        let hash_key = Key::wasm_hash(tx_code_path);

        let futures: Vec<_> = self
            .clients
            .iter()
            .map(|client| {
                rpc::query_storage_value_bytes(client, &hash_key, Some(BlockHeight(height)), false)
                    .boxed()
            })
            .collect();

        let res = self
            .concurrent_requests(futures.into_iter().map(Box::pin).collect())
            .await;

        if let Some(tx_code_bytes) = res.context("Should be able to get tx code")?.0 {
            Ok(Hash::try_from(&tx_code_bytes[..])
                .ok()
                .map(|hash| hash.to_string()))
        } else {
            Ok(None)
        }
    }

    pub async fn query_current_epoch(&self, block_height: Height) -> anyhow::Result<Option<Epoch>> {
        let futures: Vec<_> = self
            .clients
            .iter()
            .map(|client| rpc::query_epoch_at_height(client, block_height.into()).boxed())
            .collect();

        let res = self
            .concurrent_requests(futures.into_iter().map(Box::pin).collect())
            .await;

        res.map(|epoch| epoch.map(|epoch| epoch.0))
            .context("Should be able to get epoch")
    }

    pub async fn query_lastest_height(&self) -> anyhow::Result<u64> {
        let futures = self
            .clients
            .iter()
            .map(|client| client.latest_block())
            .collect();

        let res = self.concurrent_requests(futures).await;

        res.map(|response| response.block.header.height.into())
            .context("Should be able to query for block")
    }

    pub async fn query_count_slashes_before(&self, height: Height) -> anyhow::Result<usize> {
        // To count the slashes at height we need to get the slashes for the validators
        // at the tip and filter the slashes that happened after the target height :chefkiss:
        let pos_query = namada_sdk::queries::RPC.vp().pos();
        let futures = self
            .clients
            .iter()
            .map(|client| Box::pin(pos_query.slashes(client)))
            .collect();

        let res = self.concurrent_requests(futures).await;

        res.map(|response| {
            response
                .into_iter()
                .filter(|(_, slashes)| slashes.iter().any(|slash| slash.block_height < height))
                .count()
        })
        .context("Should be able to query for block")
    }

    pub async fn query_block(
        &self,
        block_height: Height,
        checksums: &Checksums,
        epoch: Epoch,
    ) -> anyhow::Result<Block> {
        let block_height = namada_sdk::tendermint::block::Height::try_from(block_height).unwrap();

        let events_futures = self
            .clients
            .iter()
            .map(|client| client.block_results(block_height))
            .collect();

        let block_height = namada_sdk::tendermint::block::Height::try_from(block_height).unwrap();

        let events_res = self.concurrent_requests(events_futures).await;
        let events = events_res.map(BlockResult::from).context(format!(
            "Should be able to query for block events for height: {}",
            block_height
        ))?;

        let block_futures = self
            .clients
            .iter()
            .map(|client| client.block(block_height))
            .collect();

        let block_res = self.concurrent_requests(block_futures).await;
        block_res
            .map(|response| Block::from(response, events, checksums, epoch))
            .context(format!(
                "Should be able to query for block for height: {}",
                block_height
            ))
    }

    pub async fn query_validator_state(
        &self,
        validator: &NamadaAddress,
        epoch: Epoch,
    ) -> anyhow::Result<ValidatorState> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::get_validator_state(client, validator, Some(epoch.into())).boxed())
            .collect();

        let res = self.concurrent_requests(futures).await;
        let (validator_state, _y) = res.context("Should be able to query validator state")?;

        match validator_state {
            Some(state_info) => Ok(state_info),
            None => Err(anyhow::anyhow!("Validator state not found")),
        }
    }

    pub async fn query_stake(
        &self,
        validator: &NamadaAddress,
        epoch: Epoch,
    ) -> anyhow::Result<u64> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::get_validator_stake(client, epoch.into(), validator).boxed())
            .collect();

        let res = self.concurrent_requests(futures).await;
        let stake = res.context("Should be able to query validator stake")?;

        Ok(stake.raw_amount().as_u64())
    }

    pub async fn query_validators(&self, epoch: Epoch) -> anyhow::Result<Vec<Validator>> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::get_all_validators(client, NamadaEpoch(epoch)).boxed())
            .collect();

        let res = self.concurrent_requests(futures).await;

        let validators = res.context("Should be able to query native token")?;
        let futures = validators.into_iter().map(|validator_address| {
            let self_ref = self;
            async move {
                let voting_power = self_ref.query_stake(&validator_address, epoch).await?;
                let state = self_ref
                    .query_validator_state(&validator_address, epoch)
                    .await?;
                Ok::<_, anyhow::Error>(Validator {
                    address: validator_address.to_string(),
                    voting_power,
                    state,
                })
            }
        });

        let results = futures::future::join_all(futures).await;
        let validators = results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .context("Should be able to query validator states")?;

        Ok(validators)
    }

    // Ask http://127.0.0.1:26657/net_info for peer
    // {
    //     "id": 0,
    //     "jsonrpc": "2.0",
    //     "result": {
    //         "listening": true,
    //         "listeners": [
    //         "Listener(@)"
    //         ],
    //         "n_peers": "1",
    //         "peers": [
    //         {
    //             "node_id": "5576458aef205977e18fd50b274e9b5d9014525a",
    //             "url": "tcp://5576458aef205977e18fd50b274e9b5d9014525a@95.179.155.35:26656"
    //         }
    //         ]
    //     }
    //     }
    pub async fn query_peers(&self) -> anyhow::Result<(Id, Vec<PeerInfo>)> {
        let futures = self
            .clients
            .iter()
            .map(|client| client.net_info().boxed())
            .collect();

        let res = self
            .concurrent_requests_idx(futures)
            .await
            .context("Should be able to query peers");

        match res {
            Ok((idx, info)) => {
                let peers: Vec<PeerInfo> = info.peers;
                let client = &self.clients[idx];
                let chain_id = client.status().await?.node_info.id;
                Ok((chain_id, peers))
            }
            _ => Err(anyhow::anyhow!("No peers found")),
        }
    }

    pub async fn query_native_token(&self) -> anyhow::Result<Address> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::query_native_token(client).boxed())
            .collect();

        let res = self.concurrent_requests(futures).await;

        res.context("Should be able to query native token")
            .map(|address| address.to_string())
    }

    pub async fn query_total_supply(&self, native_token: &str) -> anyhow::Result<u64> {
        let address = NamadaAddress::from_str(native_token)
            .context("Should be able to convert string to address")?;

        let futures = self
            .clients
            .iter()
            .map(|client| rpc::get_token_total_supply(client, &address).boxed())
            .collect();

        let res = self.concurrent_requests(futures).await;

        res.context("Should be able to query native token")
            .map(|amount| amount.raw_amount().as_u64())
    }

    pub async fn query_max_block_time_estimate(&self) -> anyhow::Result<u64> {
        let futures = self
            .clients
            .iter()
            .map(|client| rpc::query_max_block_time_estimate(client).boxed())
            .collect();

        let res = self.concurrent_requests(futures).await;

        res.context("Should be able to query max block time estimate")
            .map(|amount| amount.0)
    }

    pub async fn query_future_bonds_and_unbonds(&self, epoch: Epoch) -> anyhow::Result<(u64, u64)> {
        let pipeline_epoch = NamadaEpoch(epoch + 1);
        let futures = self
            .clients
            .iter()
            .map(|client| {
                rpc::enriched_bonds_and_unbonds(client, pipeline_epoch, &None, &None).boxed()
            })
            .collect();

        let res = self.concurrent_requests(futures).await;

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

    async fn concurrent_requests<T, E: std::fmt::Debug>(
        &self,
        futures: Vec<impl Future<Output = Result<T, E>> + Unpin>,
    ) -> Option<T> {
        self.concurrent_requests_idx(futures)
            .await
            .map(|(_idx, value)| value)
    }

    async fn concurrent_requests_idx<T, E: std::fmt::Debug>(
        &self,
        futures: Vec<impl Future<Output = Result<T, E>> + Unpin>,
    ) -> Option<(usize, T)> {
        let mut futures = futures;
        while !futures.is_empty() {
            let (result, index, remaining) = futures::future::select_all(futures).await;
            match result {
                Ok(value) => return Some((index, value)),
                Err(_e) => {
                    //tracing::error!("Error: {:?}", _e);
                    futures = remaining
                }
            }
        }
        None
    }
}
