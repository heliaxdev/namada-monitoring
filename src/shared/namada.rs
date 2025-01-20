use std::collections::BTreeMap;
use std::fmt::Display;

use namada_sdk::borsh::{BorshDeserialize, BorshSerializeExt};
use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::ibc::{decode_message, IbcMessage};
use namada_sdk::key::common::PublicKey;
use namada_sdk::token::Transfer;
use namada_sdk::tx::action::{Bond, ClaimRewards, Redelegation, Unbond, Withdraw};
use namada_sdk::tx::data::pos::{BecomeValidator, CommissionChange, MetaDataChange};
use namada_sdk::tx::data::TxResult;
use namada_sdk::tx::{data::compute_inner_tx_hash, either::Either, Tx};
use tendermint_rpc::endpoint::block::Response;
use tendermint_rpc::endpoint::block_results::Response as TendermintBlockResultResponse;
use std::str::FromStr;

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
    pub was_applied: bool,
    pub gas_used: u64,
    pub atomic: bool,
    pub inners: Vec<Inner>,
}

#[derive(Clone, Debug)]
pub enum InnerKind {
    Transfer(Option<Transfer>),
    IbcMsgTransfer(Option<IbcMessage<Transfer>>),
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
            InnerKind::Transfer(_) => write!(f, "transfer"),
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
                let data = if let Ok(data) = Transfer::try_from_slice(data) {
                    Some(data)
                } else {
                    None
                };
                InnerKind::Transfer(data)
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
                let data = if let Ok(data) = decode_message::<Transfer>(data) {
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
            InnerKind::Transfer(tx) => tx
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
    pub was_applied: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BlockResult {
    pub height: u64,
    pub begin_events: Vec<Event>,
    pub end_events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: EventKind,
    pub attributes: Option<TxAttributesType>,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Applied,
    Unknown,
}

impl From<&String> for EventKind {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "tx/applied" => Self::Applied,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TxAttributesType {
    TxApplied(TxApplied),
}

#[derive(Debug, Clone, Default)]
pub struct TxApplied {
    pub code: TxEventStatusCode,
    pub gas: u64,
    pub hash: String,
    pub height: u64,
    pub batch: BatchResults,
    pub info: String,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TxEventStatusCode {
    Ok,
    #[default]
    Fail,
}

impl From<&str> for TxEventStatusCode {
    fn from(value: &str) -> Self {
        match value {
            "0" | "1" => Self::Ok,
            _ => Self::Fail,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BatchResults {
    pub batch_errors: BTreeMap<String, BTreeMap<String, String>>,
    pub batch_results: BTreeMap<String, bool>,
}

impl From<TxResult<String>> for BatchResults {
    fn from(value: TxResult<String>) -> Self {
        Self {
            batch_results: value.0.iter().fold(
                BTreeMap::default(),
                |mut acc, (tx_hash, result)| {
                    let tx_id = tx_hash.to_string();
                    let result = if let Ok(result) = result {
                        result.is_accepted()
                    } else {
                        false
                    };
                    acc.insert(tx_id, result);
                    acc
                },
            ),
            batch_errors: value
                .0
                .iter()
                .fold(BTreeMap::default(), |mut acc, (tx_hash, result)| {
                    let tx_id = tx_hash.to_string();
                    let result = if let Ok(result) = result {
                        result
                            .vps_result
                            .errors
                            .iter()
                            .map(|(address, error)| (address.to_string(), error.clone()))
                            .collect()
                    } else {
                        BTreeMap::default()
                    };
                    acc.insert(tx_id, result);
                    acc
                }),
        }
    }
}

impl BatchResults {
    pub fn is_successful(&self, tx_id: &str) -> bool {
        match self.batch_results.get(tx_id) {
            Some(result) => *result,
            None => false,
        }
    }
}

impl Block {
    pub fn from(
        response: Response,
        block_results: BlockResult,
        checksums: &Checksums,
        epoch: Epoch,
    ) -> Self {
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

                    let wrapper_tx_id = wrapper_id.to_string();
                    let wrapper_tx_status = block_results.is_wrapper_tx_applied(&wrapper_tx_id);
                    let gas_used = block_results
                        .gas_used(&wrapper_tx_id)
                        .map(|gas| gas.parse::<u64>().unwrap_or_default())
                        .unwrap_or_default();
                    let atomic = tx.header().atomic;

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

                            let inner_tx_status =
                                block_results.is_inner_tx_accepted(&wrapper_tx_id, &inner_tx_id);

                            Inner {
                                id: inner_tx_id,
                                kind: tx_kind,
                                was_applied: inner_tx_status.was_applied(),
                            }
                        })
                        .collect();

                    Some(Wrapper {
                        id: wrapper_id.to_string(),
                        was_applied: wrapper_tx_status.was_applied(),
                        gas_used,
                        inners,
                        atomic,
                    })
                })
                .collect(),
        }
    }
}

impl BlockResult {
    pub fn is_wrapper_tx_applied(&self, tx_hash: &str) -> TransactionExitStatus {
        let exit_status = self
            .end_events
            .iter()
            .filter_map(|event| {
                if let Some(TxAttributesType::TxApplied(data)) = &event.attributes {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .find(|attributes| attributes.hash.eq(tx_hash))
            .map(|attributes| attributes.clone().code)
            .map(TransactionExitStatus::from);

        exit_status.unwrap_or(TransactionExitStatus::Rejected)
    }

    pub fn gas_used(&self, tx_hash: &str) -> Option<String> {
        self.end_events
            .iter()
            .filter_map(|event| {
                if let Some(TxAttributesType::TxApplied(data)) = &event.attributes {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .find(|attributes| attributes.hash.eq(tx_hash))
            .map(|attributes| attributes.gas.to_string())
    }

    pub fn is_inner_tx_accepted(
        &self,
        wrapper_hash: &str,
        inner_hash: &str,
    ) -> TransactionExitStatus {
        let exit_status = self
            .end_events
            .iter()
            .filter_map(|event| {
                if let Some(TxAttributesType::TxApplied(data)) = &event.attributes {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .find(|attributes| attributes.hash.eq(wrapper_hash))
            .map(|attributes| attributes.batch.is_successful(inner_hash))
            .map(|successful| match successful {
                true => TransactionExitStatus::Applied,
                false => TransactionExitStatus::Rejected,
            });
        exit_status.unwrap_or(TransactionExitStatus::Rejected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionExitStatus {
    Applied,
    Rejected,
}

impl TransactionExitStatus {
    pub fn was_applied(&self) -> bool {
        matches!(self, Self::Applied)
    }
}

impl Display for TransactionExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Applied => write!(f, "Applied"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

impl From<TxEventStatusCode> for TransactionExitStatus {
    fn from(value: TxEventStatusCode) -> Self {
        match value {
            TxEventStatusCode::Ok => Self::Applied,
            TxEventStatusCode::Fail => Self::Rejected,
        }
    }
}

impl From<&TendermintBlockResultResponse> for BlockResult {
    fn from(value: &TendermintBlockResultResponse) -> Self {
        BlockResult::from(value.clone())
    }
}

impl From<TendermintBlockResultResponse> for BlockResult {
    fn from(value: TendermintBlockResultResponse) -> Self {
        let begin_events = value
            .begin_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes = event.attributes.iter().fold(
                    BTreeMap::default(),
                    |mut acc, attribute| {
                        acc.insert(
                            String::from(attribute.key_str().unwrap()),
                            String::from(attribute.value_str().unwrap()),
                        );
                        acc
                    },
                );
                let attributes =
                    TxAttributesType::deserialize(&kind, &raw_attributes);
                Event { kind, attributes }
            })
            .collect::<Vec<Event>>();
        let end_events = value
            .end_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes = event.attributes.iter().fold(
                    BTreeMap::default(),
                    |mut acc, attribute| {
                        acc.insert(
                            String::from(attribute.key_str().unwrap()),
                            String::from(attribute.value_str().unwrap()),
                        );
                        acc
                    },
                );
                let attributes =
                    TxAttributesType::deserialize(&kind, &raw_attributes);
                Event { kind, attributes }
            })
            .collect::<Vec<Event>>();
        Self {
            height: value.height.value(),
            begin_events,
            end_events,
        }
    }
}

impl TxAttributesType {
    pub fn deserialize(
        event_kind: &EventKind,
        attributes: &BTreeMap<String, String>,
    ) -> Option<Self> {
        match event_kind {
            EventKind::Unknown => None,
            EventKind::Applied => Some(Self::TxApplied(TxApplied {
                code: attributes
                    .get("code")
                    .map(|code| TxEventStatusCode::from(code.as_str()))
                    .unwrap()
                    .to_owned(),
                gas: attributes
                    .get("gas_used")
                    .map(|gas| u64::from_str(gas).unwrap())
                    .unwrap()
                    .to_owned(),
                hash: attributes
                    .get("hash")
                    .map(|hash| hash.to_lowercase())
                    .unwrap()
                    .to_owned(),
                height: attributes
                    .get("height")
                    .map(|height| u64::from_str(height).unwrap())
                    .unwrap()
                    .to_owned(),
                batch: attributes
                    .get("batch")
                    .map(|batch_result| {
                        let tx_result: TxResult<String> =
                            serde_json::from_str(batch_result).unwrap();
                        BatchResults::from(tx_result)
                    })
                    .unwrap(),
                info: attributes.get("info").unwrap().to_owned(),
            })),
        }
    }
}