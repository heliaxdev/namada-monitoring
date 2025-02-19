use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::ibc::{self, IbcMessage};
use namada_sdk::key::common::PublicKey;
use namada_sdk::tendermint::block::Block as TendermintBlock;
use namada_sdk::uint::Uint;
use std::collections::BTreeMap;
use std::fmt::Display;

use namada_sdk::token::Transfer as NamadaTransfer;
use namada_sdk::tx::action::{Bond, ClaimRewards, Redelegation, Unbond, Withdraw};
use namada_sdk::tx::data::pos::{BecomeValidator, CommissionChange, MetaDataChange};
use namada_sdk::tx::data::{TxResult, TxType};
use namada_sdk::tx::{data::compute_inner_tx_hash, either::Either, Tx};
use std::str::FromStr;
use tendermint_rpc::endpoint::block::Response;
use tendermint_rpc::endpoint::block_results::Response as TendermintBlockResultResponse;

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
    pub block: TendermintBlock,
}

#[derive(Clone, Debug)]
pub struct Wrapper {
    pub id: TxId,
    pub inners: Vec<Inner>,
    pub fee: Fee,
    pub atomic: bool,
    pub status: TransactionExitStatus,
}

#[derive(Debug, Clone)]
pub struct Fee {
    pub gas: String,
    pub gas_used: u64,
    pub amount_per_gas_unit: String,
    pub gas_payer: String,
    pub gas_token: String,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum InnerKind {
    Transfer(NamadaTransfer),
    IbcMsgTransfer(IbcMessage<NamadaTransfer>),
    Bond(Bond),
    Redelegation(Redelegation),
    Unbond(Unbond),
    Withdraw(Withdraw),
    ClaimRewards(ClaimRewards),
    ProposalVote(VoteProposalData),
    InitProposal(InitProposalData),
    MetadataChange(MetaDataChange),
    CommissionChange(CommissionChange),
    RevealPk(PublicKey),
    BecomeValidator(BecomeValidator),
    DeactivateValidator(Address),
    ReactivateValidator(Address),
    UnjailValidator(Address),
    Unknown(String, Vec<u8>),
}

impl Display for InnerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerKind::Transfer(_) => write!(f, "transfer"),
            InnerKind::IbcMsgTransfer(_) => write!(f, "ibc_transfer"),
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
            InnerKind::DeactivateValidator(_) => write!(f, "deactivate_validator"),
            InnerKind::ReactivateValidator(_) => write!(f, "reactivate_validator"),
            InnerKind::UnjailValidator(_) => write!(f, "unjail_validator"),
            InnerKind::Unknown(code_name, ..) => write!(f, "unknown({})", code_name),
        }
    }
}

impl InnerKind {
    pub fn from(tx_code_name: &str, data: &[u8]) -> Self {
        let default = |_| InnerKind::Unknown(tx_code_name.into(), data.to_vec());
        match tx_code_name {
            "tx_transfer" => {
                NamadaTransfer::try_from_slice(data).map_or_else(default, InnerKind::Transfer)
            }
            "tx_bond" => Bond::try_from_slice(data).map_or_else(default, InnerKind::Bond),
            "tx_redelegate" => Redelegation::try_from_slice(data)
                .map_or_else(default, |redelegation| {
                    InnerKind::Redelegation(redelegation)
                }),
            "tx_unbond" => Unbond::try_from_slice(data)
                .map_or_else(default, |unbond| InnerKind::Unbond(Unbond::from(unbond))),
            "tx_withdraw" => {
                Withdraw::try_from_slice(data).map_or_else(default, InnerKind::Withdraw)
            }
            "tx_claim_rewards" => ClaimRewards::try_from_slice(data)
                .map_or_else(default, |claim_rewards| {
                    InnerKind::ClaimRewards(claim_rewards)
                }),
            "tx_init_proposal" => InitProposalData::try_from_slice(data)
                .map_or_else(default, |init_proposal| {
                    InnerKind::InitProposal(init_proposal)
                }),
            "tx_vote_proposal" => VoteProposalData::try_from_slice(data)
                .map_or_else(default, |vote_proposal| {
                    InnerKind::ProposalVote(vote_proposal)
                }),
            "tx_change_validator_metadata" => MetaDataChange::try_from_slice(data)
                .map_or_else(default, |metadata_change| {
                    InnerKind::MetadataChange(metadata_change)
                }),
            "tx_commission_change" => CommissionChange::try_from_slice(data)
                .map_or_else(default, |commission_change| {
                    InnerKind::CommissionChange(commission_change)
                }),
            "tx_reveal_pk" => {
                PublicKey::try_from_slice(data).map_or_else(default, InnerKind::RevealPk)
            }
            "tx_deactivate_validator" => {
                Address::try_from_slice(data).map_or_else(default, InnerKind::DeactivateValidator)
            }
            "tx_reactivate_validator" => {
                Address::try_from_slice(data).map_or_else(default, InnerKind::ReactivateValidator)
            }
            "tx_unjail_validator" => {
                Address::try_from_slice(data).map_or_else(default, InnerKind::UnjailValidator)
            }
            "tx_become_validator" => BecomeValidator::try_from_slice(data)
                .map_or_else(default, |become_validator| {
                    InnerKind::BecomeValidator(become_validator)
                }),

            "tx_ibc" => {
                if let Ok(ibc_data) = ibc::decode_message::<NamadaTransfer>(data) {
                    InnerKind::IbcMsgTransfer(ibc_data)
                } else {
                    InnerKind::Unknown(tx_code_name.into(), data.to_vec())
                }
            }
            _ => {
                tracing::warn!("Unknown transaction kind: {}", tx_code_name);
                InnerKind::Unknown(tx_code_name.into(), data.to_vec())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inner {
    pub id: TxId,
    pub size: usize,
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
        let block = response.block.clone();
        Self {
            block,
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

                    let fee = if let TxType::Wrapper(wrapper) = tx.header().tx_type {
                        Fee {
                            gas: Uint::from(wrapper.gas_limit).to_string(),
                            gas_used,
                            amount_per_gas_unit: wrapper
                                .fee
                                .amount_per_gas_unit
                                .to_string_precise(),
                            gas_payer: wrapper.fee_payer().to_string(),
                            gas_token: wrapper.fee.token.to_string(),
                        }
                    } else {
                        return None;
                    };

                    let inners = tx
                        .header()
                        .batch
                        .into_iter()
                        .map(|tx_commitment| {
                            let tx_id = compute_inner_tx_hash(
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
                            let tx_size = tx_data.len();

                            let tx_code_name = match tx_code_id {
                                Some(id) => checksums
                                    .get_name_by_id(&id)
                                    .unwrap_or_else(|| format!("no_tx_code_name_with_id_{}", id)),
                                None => "no_tx_id".into(),
                            };

                            let kind = InnerKind::from(&tx_code_name, &tx_data);
                            let inner_tx_id = compute_inner_tx_hash(
                                Some(&wrapper_id),
                                Either::Right(&tx_commitment),
                            )
                            .to_string();
                            let inner_tx_status =
                                block_results.is_inner_tx_accepted(&wrapper_tx_id, &inner_tx_id);

                            Inner {
                                id: tx_id,
                                size: tx_size,
                                kind,
                                was_applied: inner_tx_status.was_applied(),
                            }
                        })
                        .collect();

                    Some(Wrapper {
                        id: wrapper_id.to_string(),
                        inners,
                        fee,
                        atomic,
                        status: wrapper_tx_status,
                    })
                })
                .collect(),
        }
    }

    pub fn get_all_transfers(&self) -> Vec<Transfer> {
        let mut transfers = Vec::new();
        for tx in self
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
                match &inner.kind {
                    InnerKind::Transfer(transfer) => {
                        let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
                        for (a, b) in &transfer.targets {
                            groups
                                .entry(a.token.to_string())
                                .or_default()
                                .push(b.amount().raw_amount().as_u64());
                        }
                        for (token, amounts) in groups {
                            let total: u64 = amounts.iter().sum();
                            transfers.push(Transfer {
                                height: self.height,
                                id: inner.id.clone(),
                                kind: TransferKind::Native,
                                token: token.clone(),
                                amount: total,
                                accepted: inner.was_applied,
                            });
                        }
                    }
                    InnerKind::IbcMsgTransfer(IbcMessage::Transfer(msg_transfer)) => {
                        if let Some(transfer) = &msg_transfer.transfer {
                            let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
                            for (a, b) in &transfer.targets {
                                groups
                                    .entry(a.token.to_string())
                                    .or_default()
                                    .push(b.amount().raw_amount().as_u64());
                            }
                            for (token, amounts) in groups {
                                let total: u64 = amounts.iter().sum();
                                transfers.push(Transfer {
                                    height: self.height,
                                    id: inner.id.clone(),
                                    kind: TransferKind::Native,
                                    token: token.clone(),
                                    amount: total,
                                    accepted: inner.was_applied,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        transfers
    }
}

#[derive(Clone, Debug)]
pub enum TransferKind {
    Ibc,
    Native,
    Shielding,
    Unshielding,
}

#[derive(Clone, Debug)]
pub struct Transfer {
    pub height: Height,
    pub id: String,
    pub kind: TransferKind,
    pub token: String,
    pub amount: u64,
    pub accepted: bool,
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
                let raw_attributes =
                    event
                        .attributes
                        .iter()
                        .fold(BTreeMap::default(), |mut acc, attribute| {
                            acc.insert(
                                String::from(attribute.key_str().unwrap()),
                                String::from(attribute.value_str().unwrap()),
                            );
                            acc
                        });
                let attributes = TxAttributesType::deserialize(&kind, &raw_attributes);
                Event { kind, attributes }
            })
            .collect::<Vec<Event>>();
        let end_events = value
            .end_block_events
            .unwrap_or_default()
            .iter()
            .map(|event| {
                let kind = EventKind::from(&event.kind);
                let raw_attributes =
                    event
                        .attributes
                        .iter()
                        .fold(BTreeMap::default(), |mut acc, attribute| {
                            acc.insert(
                                String::from(attribute.key_str().unwrap()),
                                String::from(attribute.value_str().unwrap()),
                            );
                            acc
                        });
                let attributes = TxAttributesType::deserialize(&kind, &raw_attributes);
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
