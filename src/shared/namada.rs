use std::collections::BTreeMap;
use std::fmt::Display;

use namada_sdk::borsh::{BorshDeserialize, BorshSerializeExt};
use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::ibc::{decode_message, IbcMessage};
use namada_sdk::key::common::PublicKey;
use namada_sdk::masp::ShieldedTransfer;
use namada_sdk::token::Transfer as NamadaTransfer;
use namada_sdk::tx::action::{Bond, ClaimRewards, Redelegation, Unbond, Withdraw};
use namada_sdk::tx::data::pos::{BecomeValidator, CommissionChange, MetaDataChange};
use namada_sdk::tx::{data::compute_inner_tx_hash, either::Either, Tx};
use tendermint_rpc::endpoint::block::Response;

use super::checksums::Checksums;

pub type Height = u64;
pub type Epoch = u64;
pub type TxId = String;
pub type Address = String;

#[derive(Clone, Debug)]
pub struct Validator {
    pub address: String,
    pub voting_power: u64,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub height: Height,
    pub epoch: Epoch,
    pub timestamp: i64,
    pub transactions: Vec<Wrapper>,
}

#[derive(Clone, Debug)]
pub struct Wrapper {
    pub id: TxId,
    pub inners: Vec<Inner>,
}

#[derive(Clone, Debug)]
pub enum InnerKind {
    TransparentTransfer(Option<NamadaTransfer>),
    ShieldedTransfer(Option<ShieldedTransfer>),
    IbcMsgTransfer(Option<IbcMessage<NamadaTransfer>>),
    Bond(Option<Bond>),
    Redelegation(Option<Redelegation>),
    Unbond(Option<Unbond>),
    Withdraw(Option<Withdraw>),
    ClaimRewards(Option<ClaimRewards>),
    ProposalVote(Option<VoteProposalData>),
    InitProposal(Option<InitProposalData>),
    MetadataChange(Option<MetaDataChange>),
    CommissionChange(Option<CommissionChange>),
    RevealPk(Option<PublicKey>),
    BecomeValidator(Option<BecomeValidator>),
    Unknown(Vec<u8>),
}

impl Display for InnerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerKind::TransparentTransfer(_) => write!(f, "transparent_transfer"),
            InnerKind::ShieldedTransfer(_) => write!(f, "shielded_transfer"),
            InnerKind::IbcMsgTransfer(_) => write!(f, "ibc"),
            InnerKind::Bond(_) => write!(f, "bond"),
            InnerKind::Redelegation(_) => write!(f, "redelegate"),
            InnerKind::Unbond(_) => write!(f, "unbond"),
            InnerKind::Withdraw(_) => write!(f, "withdraw"),
            InnerKind::ClaimRewards(_) => write!(f, "claim_rewards"),
            InnerKind::ProposalVote(_) => write!(f, "vote_proposal"),
            InnerKind::InitProposal(_) => write!(f, "init_proposal"),
            InnerKind::MetadataChange(_) => write!(f, "metadata_change"),
            InnerKind::CommissionChange(_) => write!(f, "commission_change"),
            InnerKind::RevealPk(_) => write!(f, "reveal_public_key"),
            InnerKind::BecomeValidator(_) => write!(f, "become_validator"),
            InnerKind::Unknown(_) => write!(f, "unknown"),
        }
    }
}

impl InnerKind {
    pub fn from_code_name(name: &str, data: &[u8]) -> Self {
        match name {
            "tx_transfer" => {
                let data = if let Ok(data) = NamadaTransfer::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::TransparentTransfer(data)
            }
            "tx_bond" => {
                let data = if let Ok(data) = Bond::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::Bond(data)
            }
            "tx_redelegate" => {
                let data = if let Ok(data) = Redelegation::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::Redelegation(data)
            }
            "tx_unbond" => {
                let data = if let Ok(data) = Unbond::try_from_slice(data) {
                    Some(Unbond::from(data))
                } else {
                    None
                };
                InnerKind::Unbond(data)
            }
            "tx_withdraw" => {
                let data = if let Ok(data) = Withdraw::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::Withdraw(data)
            }
            "tx_claim_rewards" => {
                let data = if let Ok(data) = ClaimRewards::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::ClaimRewards(data)
            }
            "tx_init_proposal" => {
                let data = if let Ok(data) = InitProposalData::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::InitProposal(data)
            }
            "tx_vote_proposal" => {
                let data = if let Ok(data) = VoteProposalData::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::ProposalVote(data)
            }
            "tx_change_validator_metadata" => {
                let data = if let Ok(data) = MetaDataChange::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::MetadataChange(data)
            }
            "tx_commission_change" => {
                let data = if let Ok(data) = CommissionChange::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::CommissionChange(data)
            }
            "tx_reveal_pk" => {
                let data = if let Ok(data) = PublicKey::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::RevealPk(data)
            }
            "tx_ibc" => {
                let data = if let Ok(data) = decode_message::<NamadaTransfer>(data) {
                    Some(data)
                } else {
                    tracing::warn!("Cannot deserialize IBC transfer");
                    None
                };
                InnerKind::IbcMsgTransfer(data)
            }
            "tx_become_validator" => {
                let data = if let Ok(data) = BecomeValidator::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::BecomeValidator(data)
            }
            _ => InnerKind::Unknown(data.to_vec()),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            InnerKind::TransparentTransfer(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::ShieldedTransfer(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::IbcMsgTransfer(tx) => tx
                .clone()
                .map(|data| match data {
                    IbcMessage::Envelope(msg_envelope) => msg_envelope.serialize_to_vec().len(),
                    IbcMessage::Transfer(msg_transfer) => msg_transfer.serialize_to_vec().len(),
                    IbcMessage::NftTransfer(msg_nft_transfer) => {
                        msg_nft_transfer.serialize_to_vec().len()
                    }
                })
                .unwrap_or(0),
            InnerKind::Bond(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::Redelegation(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::Unbond(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::Withdraw(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::ClaimRewards(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::ProposalVote(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::InitProposal(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::MetadataChange(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::CommissionChange(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::RevealPk(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::BecomeValidator(tx) => tx
                .clone()
                .map(|data| data.serialize_to_vec().len())
                .unwrap_or(0),
            InnerKind::Unknown(tx) => tx.len(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inner {
    pub id: TxId,
    pub kind: InnerKind,
}

impl Block {
    pub fn from(response: Response, checksums: &Checksums, epoch: Epoch) -> Self {
        Self {
            height: response.block.header.height.value(),
            epoch,
            timestamp: response.block.header.time.unix_timestamp(),
            transactions: response
                .block
                .data
                .into_iter()
                .filter_map(|bytes| {
                    let tx = Tx::try_from(bytes.as_ref()).ok()?;
                    let wrapper_id = tx.header_hash();

                    let inners = tx
                        .header()
                        .batch
                        .into_iter()
                        .enumerate()
                        .map(|(_index, tx_commitment)| {
                            let inner_tx_id = compute_inner_tx_hash(
                                Some(&wrapper_id),
                                Either::Right(&tx_commitment),
                            )
                            .to_string();

                            let tx_code_id = tx
                                .get_section(tx_commitment.code_sechash())
                                .and_then(|s| s.code_sec())
                                .map(|s| s.code.hash().0)
                                .map(|bytes| {
                                    String::from_utf8(subtle_encoding::hex::encode(bytes)).unwrap()
                                });

                            let tx_data = tx.data(&tx_commitment).unwrap_or_default();

                            let tx_kind = if let Some(id) = tx_code_id {
                                if let Some(tx_code_name) = checksums.get_name_by_id(&id) {
                                    InnerKind::from_code_name(&tx_code_name, &tx_data)
                                } else {
                                    InnerKind::Unknown(tx_data)
                                }
                            } else {
                                InnerKind::Unknown(tx_data)
                            };

                            Inner {
                                id: inner_tx_id,
                                kind: tx_kind,
                            }
                        })
                        .collect();

                    Some(Wrapper {
                        id: wrapper_id.to_string(),
                        inners,
                    })
                })
                .collect(),
        }
    }

    pub fn get_all_transfers(&self) -> Vec<Transfer> {
        let inners = self
            .transactions
            .iter()
            .flat_map(|tx| tx.inners.clone());
        inners
            .filter_map(|inner| match inner.kind {
                InnerKind::TransparentTransfer(transfer) => {
                    if let Some(data) = transfer {
                        let groups: BTreeMap<String, Vec<u64>> =
                            data.targets
                                .into_iter()
                                .fold(BTreeMap::new(), |mut acc, (a, b)| {
                                    acc.entry(a.token.to_string())
                                        .or_default()
                                        .push(b.amount().raw_amount().as_u64());
                                    acc
                                });

                        Some(
                            groups
                                .iter()
                                .map(|(token, amounts)| {
                                    let total = amounts.iter().sum();
                                    Transfer {
                                        height: self.height,
                                        id: inner.id.clone(),
                                        kind: TransferKind::Native,
                                        token: token.clone(),
                                        amount: total,
                                    }
                                })
                                .collect(),
                        )
                    } else {
                        None
                    }
                }
                InnerKind::IbcMsgTransfer(ibc_message) => {
                    if let Some(data) = ibc_message {
                        match data {
                            IbcMessage::Transfer(msg_transfer) => {
                                if let Some(transfer) = msg_transfer.transfer {
                                    let groups: BTreeMap<String, Vec<u64>> = transfer
                                        .targets
                                        .into_iter()
                                        .fold(BTreeMap::new(), |mut acc, (a, b)| {
                                            acc.entry(a.token.to_string())
                                                .or_default()
                                                .push(b.amount().raw_amount().as_u64());
                                            acc
                                        });
                                    let transfers = groups
                                        .iter()
                                        .map(|(token, amounts)| {
                                            let total = amounts.iter().sum();
                                            Transfer {
                                                height: self.height,
                                                id: inner.id.clone(),
                                                kind: TransferKind::Native,
                                                token: token.clone(),
                                                amount: total,
                                            }
                                        })
                                        .collect::<Vec<Transfer>>();
                                    Some(transfers)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flatten()
            .collect()
    }
}

pub enum TransferKind {
    Ibc,
    Native,
}

pub struct Transfer {
    pub height: Height,
    pub id: String,
    pub kind: TransferKind,
    pub token: String,
    pub amount: u64,
}
