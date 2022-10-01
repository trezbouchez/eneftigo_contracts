use crate::*;

// Deposits
pub const NO_DEPOSIT: Balance = 0;
pub const MIN_DEPOSIT: Balance = 100_000_000_000_000_000_000_000;     // 0.1 Near

#[allow(dead_code)]
pub const ACCOUNT_NAME_LEN_MAX: usize = 64;     //https://nomicon.io/DataStructures/Account

/* NFT contract cross call constants */
// TODO: measure them, they're overshoot for sure

// pub const NFT_MINT_GAS: Gas = Gas(15_000_000_000_000);              
// pub const NFT_MINT_COMPLETION_GAS: Gas = Gas(5_000_000_000_000);
// minimum balance needed by an NFT account
//pub const NFT_NAKED_ACCOUNT_REQUIRED_BALANCE: Balance = 1_000_000_000_000_000_000_000;
// pub const NFT_CONTRACT_STORAGE_COST: Balance = 3_154_650_000_000_000_000_000_000

// GAS constants to attach to calls
// const GAS_FOR_ROYALTIES: Gas = Gas(115_000_000_000_000);
// const GAS_FOR_NFT_TRANSFER: Gas = Gas(15_000_000_000_000);
// pub const GAS_FOR_NFT_DEPLOY: Gas = Gas(1_000_000_000_000_000_000);
