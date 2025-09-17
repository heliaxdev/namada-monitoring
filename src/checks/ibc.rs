use std::str::FromStr;
use std::time::Duration;

use namada_sdk::ibc::context::client::{AnyClientState, AnyConsensusState};
use namada_sdk::ibc::core::host::types::path::ClientConsensusStatePath;
use namada_sdk::ibc::core::host::types::{identifiers::ClientId, path::ClientStatePath};
use namada_sdk::ibc::primitives::proto::Any;
use namada_sdk::ibc::storage;
use namada_sdk::rpc;
use prost::Message;
use tendermint_rpc::HttpClient;

use crate::shared::alert::Metadata;

use super::{AppConfig, CheckTrait};

const HEALTHY_IBC_THRESHOLD: i64 = 60 * 60 * 24 * 2; // 2 days

const IBC_CHECK_ID: &str = "ibc_check";

struct IbcChannel {
    pub channel_id: u64,
    pub connection_id: u64,
    pub client_id: u64,
    pub alias: String,
}

pub struct IbcCheck {
    channels: Vec<IbcChannel>,
    rpc: String,
}

#[async_trait::async_trait]
impl CheckTrait for IbcCheck {
    async fn check(&self, _state: &crate::state::State) -> Vec<crate::shared::alert::Alert> {
        let mut alerts = vec![];

        let client = HttpClient::new(self.rpc.as_str()).unwrap();
        for channel in self.channels.iter() {
            let client_id = format!("07-tendermint-{}", channel.client_id);
            let client_state_path =
                ClientStatePath(ClientId::from_str(&client_id).expect("valid client ID"));
            let client_state_key = storage::ibc_key(client_state_path.to_string())
                .expect("the path should be parsable");
            let client_state_res =
                rpc::query_storage_value_bytes(&client, &client_state_key, None, false).await;

            let client_state = match client_state_res {
                Ok((Some(value), _proof)) => {
                    let any = Any::decode(value.as_slice()).expect("decode Any");
                    let AnyClientState::Tendermint(client_state) =
                        AnyClientState::try_from(any).expect("valid client state");

                    client_state.inner().clone()
                }
                Ok((None, _proof)) => {
                    tracing::warn!(
                        "No IBC client state found for {} - channel {} - connection {} - client {}",
                        channel.alias,
                        channel.channel_id,
                        channel.connection_id,
                        channel.client_id
                    );
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to query IBC client state for {}: {}",
                        channel.alias,
                        e
                    );
                    continue;
                }
            };

            let client_consensus_state_path = ClientConsensusStatePath {
                client_id: ClientId::from_str(&client_id).expect("valid client ID"),
                revision_number: client_state.latest_height.revision_number(),
                revision_height: client_state.latest_height.revision_height(),
            };

            let client_consensus_key = storage::ibc_key(client_consensus_state_path.to_string())
                .expect("the path should be parsable");
            let client_consensus_state_res =
                rpc::query_storage_value_bytes(&client, &client_consensus_key, None, false).await;

            let client_consensus_state = match client_consensus_state_res {
                Ok((Some(value), _proof)) => {
                    let any = Any::decode(value.as_slice()).expect("decode Any");
                    let AnyConsensusState::Tendermint(client_consensus_state) =
                        AnyConsensusState::try_from(any).expect("valid client state");

                    client_consensus_state.inner().clone()
                }
                Ok((None, _proof)) => {
                    tracing::warn!(
                        "No IBC client state found for {} - channel {} - connection {} - client {}",
                        channel.alias,
                        channel.channel_id,
                        channel.connection_id,
                        channel.client_id
                    );
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to query IBC client state for {}: {}",
                        channel.alias,
                        e
                    );
                    continue;
                }
            };

            let expiration_time = client_consensus_state.timestamp.unix_timestamp()
                + client_state.trusting_period.as_secs() as i64;
            let expiration_timestamp = expiration_time as f64;
            let now = chrono::Utc::now().timestamp() as f64;

            if now + HEALTHY_IBC_THRESHOLD as f64 > expiration_timestamp {
                alerts.push(crate::shared::alert::Alert {
                    check_id: format!("{}-{}", IBC_CHECK_ID, channel.alias),
                    title: "IBC Client Expiration".to_string(),
                    description: format!("The IBC channel {} for *{}* (connection {}, client {}), is near expiration: *{}* seconds left.", channel.channel_id, channel.alias, channel.connection_id, channel.client_id, (expiration_timestamp - now) / 3600.0),
                    severity: crate::shared::alert::Severity::High,
                    metadata: Metadata::new(
                        None, None
                    ),
                    trigger_after: Some(Duration::from_secs(60 * 60)),
                    continous: self.is_continous(),
                });
            };
        }

        alerts
    }

    fn is_continous(&self) -> bool {
        true
    }
}

impl IbcCheck {
    pub fn new(config: &AppConfig) -> Self {
        let channels = config
            .get_config()
            .ibcs
            .iter()
            .map(|ibc| IbcChannel {
                channel_id: ibc.channel,
                connection_id: ibc.connection,
                alias: ibc.alias.clone(),
                client_id: ibc.client_id,
            })
            .collect();

        IbcCheck {
            channels,
            rpc: config.rpc.clone(),
        }
    }
}
