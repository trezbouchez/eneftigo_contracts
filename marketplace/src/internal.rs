use crate::*;
use near_sdk::CryptoHash;
// use near_sdk::collections::Vector;

// used to generate a unique prefix in our storage collections (this is to avoid data collisions)
pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

impl MarketplaceContract {

    pub(crate) fn internal_nft_shared_contract_id(&mut self) -> AccountId {
        AccountId::new_unchecked(format!("nft.{}", env::current_account_id()))
    }

    pub(crate) fn fees_account_id(&mut self) -> AccountId {
        AccountId::new_unchecked(format!("fees.{}", env::current_account_id()))
    }
}
