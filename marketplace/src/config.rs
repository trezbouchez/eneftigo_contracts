use crate::*;

// Non-Fungible Token Id
pub type TokenId = String;

//constant used to attach 0 NEAR to a call
pub const NO_DEPOSIT: Balance = 0;

// GAS constants to attach to calls
// const GAS_FOR_ROYALTIES: Gas = Gas(115_000_000_000_000);
// const GAS_FOR_NFT_TRANSFER: Gas = Gas(15_000_000_000_000);
pub const GAS_FOR_NFT_MINT: Gas = Gas(15_000_000_000_000);
