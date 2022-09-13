use workspaces::types::{Gas};

/*
    Gas
*/
pub const FPO_BUY_NOW_ONLY_ADD_GAS: Gas = 50_000_000_000_000;   // TODO: measure
pub const FPO_BUY_NOW_ONLY_BUY_GAS: Gas = 100_000_000_000_000;  // TODO: measure
pub const FPO_BUY_NOW_ONLY_CONCLUDE_GAS: Gas = 10_000_000_000_000; // actual measured was 7_942_179_600_919
pub const FPO_ACCEPTING_PROPOSALS_ADD_GAS: Gas = 50_000_000_000_000;   // TODO: measure
pub const FPO_ACCEPTING_PROPOSALS_PLACE_GAS: Gas = 10_000_000_000_000;   // TODO: measure (with a lot of proposals!)
pub const FPO_ACCEPTING_PROPOSALS_BUY_GAS: Gas = 100_000_000_000_000;    // TODO: measure (worst case - when outbidding proposal)
pub const FPO_ACCEPTING_PROPOSALS_CONCLUDE_GAS: Gas = 10_000_000_000_000;    // TODO: measure when there's a lot of proposers 
pub const FPO_ACCEPTING_PROPOSALS_REVOKE_GAS: Gas = 10_000_000_000_000; // TODO: measure

/*
    Storage
*/
pub const FPO_ADD_WORST_CASE_MARKETPLACE_STORAGE: u64 = 1671; // actual measured was 1349
pub const NEW_COLLECTION_WORST_CASE_NFT_STORAGE: u64 = 4376; // actual measured for worst case title and url
pub const FPO_ACCEPTING_PROPOSALS_PLACE_STORAGE: u64 = 796;     // actual measured was 796

pub const FPO_ADD_WORST_CASE_STORAGE: u64 =
    FPO_ADD_WORST_CASE_MARKETPLACE_STORAGE + NEW_COLLECTION_WORST_CASE_NFT_STORAGE;
pub const NFT_MINT_WORST_CASE_STORAGE: u64 = 830; // actual measured was 830

/*
    Fees
*/
pub const FPO_ACCEPTING_PROPOSALS_REVOKE_FEE_RATE: u128 = 10u128; 

/*
    Other
*/
// pub const ACCOUNT_NAME_LEN_MAX: usize = 64; //https://nomicon.io/DataStructures/Account
pub const STORAGE_COST_YOCTO_PER_BYTE: u128 = 10000000000000000000;

/*
    Wasm Paths
*/
pub const MARKETPLACE_WASM_FILEPATH: &str = "../out/marketplace.wasm";
pub const NFT_WASM_FILEPATH: &str = "../out/nft.wasm";

pub const MIN_DURATION_SECS: i64 = 3600; // 1 hour
pub const MAX_DURATION_SECS: i64 = 3600 * 24 * 14; // 2 weeks
