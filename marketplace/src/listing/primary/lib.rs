use crate::*;
use crate::external::{NftMetadata};
use super::super::{
    bid::{Bid},
    status::{ListingStatus},
};
use std::fmt;
use near_sdk::collections::Vector;
use near_sdk::json_types::{U64};

#[derive(BorshStorageKey, BorshSerialize)]
pub enum PrimaryListingStorageKey {
    Bids {
        listing_id_hash: CryptoHash,
    },
}

#[derive(BorshDeserialize, BorshSerialize)]
// #[derive(Serialize,Deserialize)]
// #[serde(crate = "near_sdk::serde")]
#[derive(Clone)]
pub struct PrimaryListingId {
    pub nft_contract_id: AccountId,
    pub collection_id: u64,
}

impl fmt::Display for PrimaryListingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ nft_contract_id: {}, collection_id: {}}}",
            self.nft_contract_id, self.collection_id,
        )
    }
}

#[derive(Serialize,Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PrimaryListingIdJson {
    pub nft_contract_id: AccountId,
    pub collection_id: U64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PrimaryListing {
    pub id: PrimaryListingId,
    pub seller_id: AccountId,
    pub nft_metadata: NftMetadata,
    pub supply_total: u64,
    pub price_yocto: Option<u128>,              // if None, then it's an auction without buy now price
    pub min_bid_yocto: Option<u128>,            // if None then no bids will be accepted
    pub start_timestamp: i64,                   // nanoseconds since 1970-01-01
    pub end_timestamp: Option<i64>,             // nanoseconds since 1970-01-01
    pub status: ListingStatus,                  // will be updated when any buyer transaction is mined
    pub supply_left: u64,
    pub bids: Vector<Bid>,
    pub next_bid_id: u64,
}

impl fmt::Display for PrimaryListing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(nft_contract_id: {}\n, collection_id: {}\n, seller_id{}\n)",
            self.id.nft_contract_id, self.id.collection_id, self.seller_id
        )
    }
}
