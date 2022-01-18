use crate::*;
use near_sdk::{CryptoHash};

//used to generate a unique prefix in our storage collections (this is to avoid data collisions)
pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

impl MarketplaceContract {

    //add FPO to the set of fpos an offeror offered
    pub(crate) fn internal_add_fpo_to_offeror(
        &mut self,
        offeror_id: &AccountId,
        nft_contract_id: &AccountId,
    ) {
        //get the set of FPOs for the given owner account
        let mut fpo_set = self.fpos_by_offeror_id.get(offeror_id).unwrap_or_else(|| {
            //if the offeror doesn't have any fpos yet we'll create the new unordered set
            UnorderedSet::new(
                StorageKey::FPOsByOfferorIdInner {
                    account_id_hash: hash_account_id(&offeror_id),  // generate a new unique prefix for the collection
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        // insert the nft_account_id into the set
        fpo_set.insert(nft_contract_id);

        // insert back
        self.fpos_by_offeror_id.insert(offeror_id, &fpo_set);
    }
}