use crate::*;
use near_sdk::StorageUsage;

// Non-Fungible Token Id
// pub type TokenId = String;

//constant used to attach 0 NEAR to a call
pub const NO_DEPOSIT: Balance = 0;

/* NFT contract cross call constants */
pub const NFT_MAKE_COLLECTION_STORAGE: StorageUsage = 79;           // make_collection storage
pub const NFT_MAKE_COLLECTION_GAS: Gas = Gas(5_789_462_639_555);            // make_collection gas consumption
pub const NFT_MAKE_COLLECTION_COMPLETION_GAS: Gas = Gas(1_000_000_000_000);

// minimum balance needed by an NFT account
//pub const NFT_NAKED_ACCOUNT_REQUIRED_BALANCE: Balance = 1_000_000_000_000_000_000_000;
// pub const NFT_CONTRACT_STORAGE_COST: Balance = 3_154_650_000_000_000_000_000_000

// GAS constants to attach to calls
// const GAS_FOR_ROYALTIES: Gas = Gas(115_000_000_000_000);
// const GAS_FOR_NFT_TRANSFER: Gas = Gas(15_000_000_000_000);
// pub const GAS_FOR_NFT_DEPLOY: Gas = Gas(1_000_000_000_000_000_000);
pub const GAS_FOR_NFT_MINT: Gas = Gas(15_000_000_000_000);
