use crate::*;
use near_sdk::{CryptoHash, Balance};
// use near_sdk::collections::Vector;

// used to generate a unique prefix in our storage collections (this is to avoid data collisions)
pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

// refund the initial deposit based on the amount of storage that was used up
pub(crate) fn refund_deposit(storage_used: u64) {
    //get how much it would cost to store the information
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    //get the attached deposit
    let attached_deposit = env::attached_deposit();

    //make sure that the attached deposit is greater than or equal to the required cost
    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    //get the refund amount from the attached deposit - required cost
    let refund = attached_deposit - required_cost;

    //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

// pub(crate) fn vec_to_vector<T: near_sdk::borsh::BorshSerialize>(vec: Vec<T>, id: Vec<u8>) -> Vector<T>
// {
//     let mut vector = Vector::new(id);
//     vector.extend(vec);
//     return vector;
// }

impl MarketplaceContract {

    // doesn't check if already there!
    pub(crate) fn internal_add_fpo(
        &mut self,
        fpo: &FixedPriceOffering,
    ) {
        self.fpos_by_contract_id.insert(&fpo.nft_account_id, &fpo);
        self.internal_add_fpo_to_offeror(&fpo.offeror_id, &fpo.nft_account_id);
    }

    pub(crate) fn internal_remove_fpo(
        &mut self,
        nft_account_id: &AccountId
    ) -> FixedPriceOffering {
        let removed_fpo = self.fpos_by_contract_id.remove(&nft_account_id).expect("Could not remove FPO: Could not find FPO listing");
        let offeror_id = &removed_fpo.offeror_id;
        let fpos_by_this_offeror = &mut self.fpos_by_offeror_id.get(offeror_id).expect("Could not remove FPO: Could not find listings for offeror");
        let did_remove = fpos_by_this_offeror.remove(&nft_account_id);
        assert!(did_remove, "Could not remove FPO: Offering not on offeror's list");
        if fpos_by_this_offeror.is_empty() {
            self.fpos_by_offeror_id.remove(offeror_id).expect("Could not remove FPO: Could not remove the now-empty offeror list");
        }
        
        removed_fpo
    }

    // add FPO to the set of fpos an offeror offered
    // doesn't check if already there
    pub(crate) fn internal_add_fpo_to_offeror(
        &mut self,
        offeror_id: &AccountId,
        nft_account_id: &AccountId,
    ) {
        //get the set of FPOs for the given owner account
        let mut fpo_set = self.fpos_by_offeror_id.get(offeror_id).unwrap_or_else(|| {
            //if the offeror doesn't have any fpos yet we'll create the new unordered set
            UnorderedSet::new(
                MarketplaceStorageKey::FposByOfferorIdInner {
                    account_id_hash: hash_account_id(&offeror_id),  // generate a new unique prefix for the collection
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        // insert the nft_account_id into the set
        fpo_set.insert(nft_account_id);

        // insert back
        self.fpos_by_offeror_id.insert(offeror_id, &fpo_set);
    }

    pub(crate) fn internal_make_nft_account_id(
        &mut self,
    ) -> AccountId {
        let nft_account_id = AccountId::new_unchecked(format!(
            "nft{}.{}",
            self.nft_account_id_prefix,
            env::current_account_id()
        ));
    
        self.nft_account_id_prefix += 1;

        nft_account_id
    }
}
