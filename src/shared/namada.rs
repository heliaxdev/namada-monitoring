use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::ibc::{decode_message, IbcMessage};
use namada_sdk::key::common::PublicKey;
use namada_sdk::masp::ShieldedTransfer;
use namada_sdk::token::Transfer;
use namada_sdk::tx::action::{Bond, ClaimRewards, Redelegation, Unbond, Withdraw};
use namada_sdk::tx::data::pos::{BecomeValidator, CommissionChange, MetaDataChange};
use namada_sdk::tx::{data::compute_inner_tx_hash, either::Either, Tx};
use tendermint_rpc::endpoint::block::Response;

use super::checksums::Checksums;

pub type Height = u64;
pub type Epoch = u64;
pub type TxId = String;

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
    TransparentTransfer(Option<Transfer>),
    ShieldedTransfer(Option<ShieldedTransfer>),
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

impl InnerKind {
    pub fn from_code_name(name: &str, data: &[u8]) -> Self {
        match name {
            "tx_transfer" => {
                let data = if let Ok(data) = Transfer::try_from_slice(data) {
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
}
