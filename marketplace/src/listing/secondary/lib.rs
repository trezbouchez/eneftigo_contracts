use crate::*;
use external::{NftMetadata, NftMutableMetadata};
use super::super::{
    bid::{Bid},
    status::{ListingStatus},
};
use std::{
    fmt,
};

use near_sdk::collections::Vector;

#[derive(BorshStorageKey, BorshSerialize)]
pub enum SecondaryListingStorageKey {
    Bids {
        listing_id_hash: CryptoHash,
    },
}

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Serialize,Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Clone)]
pub struct SecondaryListingId {
    pub nft_contract_id: AccountId,
    pub token_id: NftId,
}

#[derive(Serialize,Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SecondaryListingIdJson {
    pub nft_contract_id: AccountId,
    pub token_id: NftId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SecondaryListing {
    pub id: SecondaryListingId,
    pub seller_id: AccountId,
    pub approval_id: u64,
    pub nft_metadata: NftMetadata,
    pub nft_mutable_metadata: NftMutableMetadata,
    pub price_yocto: Option<u128>,
    pub min_bid_yocto: Option<u128>,            // if None then no bids will be accepted
    pub start_timestamp: i64,                   // nanoseconds since 1970-01-01
    pub end_timestamp: Option<i64>,             // nanoseconds since 1970-01-01
    pub status: ListingStatus, // will be updated when any buyer transaction is mined
    pub bids: Vector<Bid>,
    pub next_bid_id: u64,
}

impl fmt::Display for SecondaryListing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(nft_contract_id: {}\n, seller_id{}\n)",
            self.id.nft_contract_id, self.seller_id
        )
    }
}
