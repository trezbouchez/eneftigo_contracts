use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault,
    Promise, CryptoHash, BorshStorageKey,
    json_types::{Base64VecU8},
    collections::{LookupMap, UnorderedMap, UnorderedSet},
    serde::{Deserialize, Serialize},
    borsh::{self, BorshDeserialize, BorshSerialize},
};
use listing::{
    primary::lib::{PrimaryListingId, PrimaryListing},
    secondary::lib::{SecondaryListingId, SecondaryListing},
};
use std::{
    collections::{HashMap},
};

mod listing;
mod internal;
mod enumeration;
mod external;
mod constants;
mod callback;
mod deposit;

// mod error;

pub type NftCollectionId = u64;
pub type NftId = String;

//main contract struct to store all the information
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MarketplaceContract {
    pub owner_id: AccountId,
    pub primary_listings_by_id: UnorderedMap<PrimaryListingId, PrimaryListing>,
    pub primary_listings_by_seller_id: LookupMap<AccountId, UnorderedSet<PrimaryListingId>>,
    pub secondary_listings_by_id: UnorderedMap<SecondaryListingId, SecondaryListing>,
    pub secondary_listings_by_seller_id: LookupMap<AccountId, UnorderedSet<SecondaryListingId>>,
    pub storage_deposits: LookupMap<AccountId,Balance>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum MarketplaceStorageKey {
    PrimaryListingsById,
    PrimaryListingsBySellerId,
    PrimaryListingsBySellerIdInner { account_id_hash: CryptoHash },
    SecondaryListingsById,
    SecondaryListingsBySellerId,
    SecondaryListingsBySellerIdInner { account_id_hash: CryptoHash },
    StorageDeposits,
}

#[near_bindgen]
impl MarketplaceContract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default data and the owner ID
        that's passed in
    */
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            primary_listings_by_id: UnorderedMap::new(MarketplaceStorageKey::PrimaryListingsById),
            primary_listings_by_seller_id: LookupMap::new(MarketplaceStorageKey::PrimaryListingsBySellerId),
            secondary_listings_by_id: UnorderedMap::new(MarketplaceStorageKey::SecondaryListingsById),
            secondary_listings_by_seller_id: LookupMap::new(MarketplaceStorageKey::SecondaryListingsBySellerId),
            storage_deposits: LookupMap::new(MarketplaceStorageKey::StorageDeposits),
        }
    }

    pub fn clean(keys: Vec<Base64VecU8>) {
        for key in keys.iter() {
            env::storage_remove(&key.0);
        }
    }
}

