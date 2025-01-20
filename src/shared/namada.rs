use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::ibc::IbcMessage;
use namada_sdk::key::common::PublicKey;
use namada_sdk::token::Transfer as NamadaTransfer;
use namada_sdk::tx::action::{Bond, ClaimRewards, Redelegation, Unbond, Withdraw};
use namada_sdk::tx::data::pos::{BecomeValidator, CommissionChange, MetaDataChange};
use namada_sdk::tx::{data::compute_inner_tx_hash, either::Either, Tx};
use std::fmt::Display;
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
    TransparentTransfer(NamadaTransfer),
    IbcMsgTransfer(Option<IbcMessage<NamadaTransfer>>),
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
            InnerKind::TransparentTransfer(_) => write!(f, "transfer"),
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
            InnerKind::Unknown(..) => write!(f, "unknown"),
        }
    }
}

impl InnerKind {
    pub fn from(tx_code_name: &str, data: &[u8]) -> Self {
        let default = |_| InnerKind::Unknown(tx_code_name.into(), data.to_vec());
        match tx_code_name {
            "tx_transfer" => NamadaTransfer::try_from_slice(data)
                .map_or_else(default, |data| InnerKind::TransparentTransfer(data)),
            "tx_bond" => {
                Bond::try_from_slice(data).map_or_else(default, |bond| InnerKind::Bond(bond))
            }
            "tx_redelegate" => Redelegation::try_from_slice(data)
                .map_or_else(default, |redelegation| {
                    InnerKind::Redelegation(redelegation)
                }),
            "tx_unbond" => Unbond::try_from_slice(data)
                .map_or_else(default, |unbond| InnerKind::Unbond(Unbond::from(unbond))),
            "tx_withdraw" => Withdraw::try_from_slice(data)
                .map_or_else(default, |withdraw| InnerKind::Withdraw(withdraw)),
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
                PublicKey::try_from_slice(data).map_or_else(default, |pk| InnerKind::RevealPk(pk))
            }
            "tx_deactivate_validator" => Address::try_from_slice(data)
                .map_or_else(default, |address| InnerKind::DeactivateValidator(address)),
            "tx_reactivate_validator" => Address::try_from_slice(data)
                .map_or_else(default, |address| InnerKind::ReactivateValidator(address)),
            "tx_unjail_validator" => Address::try_from_slice(data)
                .map_or_else(default, |address| InnerKind::UnjailValidator(address)),
            "tx_become_validator" => BecomeValidator::try_from_slice(data)
                .map_or_else(default, |become_validator| {
                    InnerKind::BecomeValidator(become_validator)
                }),
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

                            Inner {
                                id: tx_id,
                                size: tx_size,
                                kind,
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
        // TODO
        vec![]
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
}
