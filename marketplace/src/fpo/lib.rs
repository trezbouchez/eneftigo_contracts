use crate::*;

use std::cmp::Ordering;
use std::fmt;

use near_sdk::collections::Vector;

pub type ProposalId = u64;

// contains fixed-price offering parameters

#[derive(BorshStorageKey, BorshSerialize)]
pub enum FixedPriceOfferingStorageKey {
    Proposals {
        offering_id_hash: CryptoHash,
    },
    ProposalsByProposer {
        offering_id_hash: CryptoHash,
    },
    ProposalsByProposerInner {
        offering_id_hash: CryptoHash,
        proposer_id_hash: CryptoHash,
    },
    AcceptableProposals {
        offering_id_hash: CryptoHash,
    },
}

#[derive(BorshDeserialize, BorshSerialize /*, Serialize, Deserialize*/, Eq)]
// #[serde(crate = "near_sdk::serde")]
pub struct FixedPriceOfferingProposal {
    pub id: ProposalId,
    pub proposer_id: AccountId,
    pub price_yocto: u128,
    pub is_acceptable: bool,
}

impl Ord for FixedPriceOfferingProposal {
    // lower looses, greater wins
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price_yocto < other.price_yocto {
            // lower proposed price
            Ordering::Less
        } else if self.price_yocto == other.price_yocto {
            other.id.cmp(&self.id)
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for FixedPriceOfferingProposal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FixedPriceOfferingProposal {
    fn eq(&self, other: &Self) -> bool {
        self.price_yocto == other.price_yocto && self.id == other.id
    }
}

impl fmt::Display for FixedPriceOfferingProposal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, proposer_id: {}, price_yocto: {}, is_acceptable: {} }}",
            self.id, self.proposer_id, self.price_yocto, self.is_acceptable
        )
    }
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
pub enum FixedPriceOfferingStatus {
    Unstarted,
    Running,
    Ended,
}

impl FixedPriceOfferingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FixedPriceOfferingStatus::Unstarted => "Unstarted",
            FixedPriceOfferingStatus::Running => "Running",
            FixedPriceOfferingStatus::Ended => "Ended",
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FixedPriceOffering {
    pub offering_id: OfferingId,
    pub offeror_id: AccountId,
    pub nft_metadata: NftMetadata,
    pub supply_total: u64,
    pub buy_now_price_yocto: u128,
    pub min_proposal_price_yocto: Option<u128>, // if None then no proposals will be accepted
    pub start_timestamp: Option<i64>,           // nanoseconds since 1970-01-01
    pub end_timestamp: Option<i64>,             // nanoseconds since 1970-01-01
    pub status: FixedPriceOfferingStatus, // will be updated when any buyer transaction is mined
    // pub nft_metadata: TokenMetadata,
    pub supply_left: u64,
    pub proposals: LookupMap<ProposalId, FixedPriceOfferingProposal>,
    pub proposals_by_proposer: LookupMap<AccountId, UnorderedSet<ProposalId>>,
    pub acceptable_proposals: Vector<ProposalId>, // by ascending price then by ascending id (submission order)
    pub next_proposal_id: u64,
}

impl fmt::Display for FixedPriceOffering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(nft_contract_id: {}\n, collection_id: {}\n, offeror_id{}\n)",
            self.offering_id.nft_contract_id, self.offering_id.collection_id, self.offeror_id
        )
    }
}
