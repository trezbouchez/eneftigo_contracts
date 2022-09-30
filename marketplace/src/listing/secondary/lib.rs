use crate::*;
use external::{NftMetadata};
use super::super::{
    proposal::{Proposal},
    status::{ListingStatus},
};
use std::{
    fmt,
};

use near_sdk::collections::Vector;

// contains fixed-price offering parameters

#[derive(BorshStorageKey, BorshSerialize)]
pub enum FixedPriceSaleStorageKey {
    Proposals {
        sale_id_hash: CryptoHash,
    },
    ProposalsByProposer {
        sale_id_hash: CryptoHash,
    },
    ProposalsByProposerInner {
        sale_id_hash: CryptoHash,
        proposer_id_hash: CryptoHash,
    },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PrimaryListing {
    pub offering_id: PrimaryListingId,
    pub offeror_id: AccountId,
    pub nft_metadata: NftMetadata,
    pub supply_total: u64,
    pub buy_now_price_yocto: u128,
    pub min_proposal_price_yocto: Option<u128>, // if None then no proposals will be accepted
    pub start_timestamp: Option<i64>,           // nanoseconds since 1970-01-01
    pub end_timestamp: Option<i64>,             // nanoseconds since 1970-01-01
    pub status: ListingStatus, // will be updated when any buyer transaction is mined
    pub supply_left: u64,
    pub proposals: Vector<Proposal>,
    pub next_proposal_id: u64,
}

impl fmt::Display for PrimaryListing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(nft_contract_id: {}\n, collection_id: {}\n, offeror_id{}\n)",
            self.offering_id.nft_contract_id, self.offering_id.collection_id, self.offeror_id
        )
    }
}
