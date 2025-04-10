use crate::shared::namada::{Inner, InnerKind, Wrapper};
use ibc::clients::tendermint::types::Header as ClientHeader;
use namada_sdk::ibc::{
    self,
    core::{client::types::msgs::ClientMsg, handler::types::msgs::MsgEnvelope},
    IbcMessage,
};
use std::collections::HashSet;

use super::{CheckTrait, State};

#[derive(Default)]
pub struct TendermintRsCheck {}

impl CheckTrait for TendermintRsCheck {
    fn check(&self, states: &[&State]) -> Vec<String> {
        let last_state = states.last().unwrap();
        let last_block = last_state.get_last_block();
        let mut results = Vec::new();

        for tx in last_block
            .transactions
            .iter()
            .filter(|tx| tx.status.was_applied())
            .cloned()
            .collect::<Vec<Wrapper>>()
        {
            for inner in tx
                .inners
                .iter()
                .filter(|tx| tx.was_applied)
                .cloned()
                .collect::<Vec<Inner>>()
            {
                if let InnerKind::IbcMsgTransfer(IbcMessage::Envelope(msg_envelope)) = &inner.kind {
                    match msg_envelope.as_ref() {
                        MsgEnvelope::Client(client_msg) => {
                            match client_msg {
                                ClientMsg::CreateClient(msg_create_client) => {
                                    let header = ClientHeader::try_from(
                                        msg_create_client.consensus_state.clone(),
                                    )
                                    .unwrap();
                                    let mut address_set = HashSet::new();
                                    for val in header.validator_set.validators() {
                                        if address_set.contains(&val.address) {
                                            tracing::info!("Validator already exists in the validator set light client attack: {:?} !!!!!!!!!!!!!!!!!!!!!" , val.address);
                                            // make descriptive alert text for the detection of an attack attempt
                                            let signed_header = &header.signed_header;
                                            let alert_text = format!(
                                                "💥 Tendermint rs attack attempt detected in tx {}!
                                                Validator {:?} from chain {} is repeated in the validator set on a CreateClient IBC.",
                                                tx.id, val.address, signed_header.header.chain_id
                                            );
                                            results.push(alert_text);
                                        } else {
                                            address_set.insert(val.address);
                                        }
                                    }
                                    tracing::info!("Ibc Client created with {} validators (all different address)", address_set.len());
                                }
                                ClientMsg::UpdateClient(msg_update_client) => {
                                    let header = ClientHeader::try_from(
                                        msg_update_client.client_message.clone(),
                                    )
                                    .unwrap();
                                    let mut address_set = HashSet::new();
                                    for val in header.validator_set.validators() {
                                        if address_set.contains(&val.address) {
                                            tracing::info!("Validator already exists: {:?} !!!!!!!!!!!!!!!!!!!!!" , val.address);
                                            // make descriptive alert text for the detection of an attack attempt
                                            let signed_header = &header.signed_header;
                                            let alert_text = format!(
                                                "💥 Tendermint rs attack attempt detected in tx {}!
                                                Validator {:?} from chain {} is repeated in the validator set on an UpdateClient IBC.",
                                                tx.id, val.address, signed_header.header.chain_id
                                            );
                                            results.push(alert_text);
                                        } else {
                                            address_set.insert(val.address);
                                        }
                                    }
                                    tracing::info!("Ibc Client updated with {} validators (all different address)", address_set.len());
                                }
                                ClientMsg::Misbehaviour(_msg_submit_misbehaviour) => {}
                                ClientMsg::UpgradeClient(_msg_upgrade_client) => {}
                                ClientMsg::RecoverClient(_msg_recover_client) => {}
                            }
                        }
                        MsgEnvelope::Connection(_connection_msg) => {}
                        MsgEnvelope::Channel(_channel_msg) => {}
                        MsgEnvelope::Packet(_packet_msg) => {}
                    }
                }
            }
        }
        results
    }
}
