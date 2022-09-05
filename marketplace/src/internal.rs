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

pub(crate) fn hash_offering_id(offering_id: &OfferingId) -> CryptoHash {
    //get the default hash
    //we hash the account ID and return it
    let hashed_string = format!("{}.{}", offering_id.nft_contract_id, offering_id.collection_id);
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(hashed_string.as_bytes()));
    hash
}

impl MarketplaceContract {
    // doesn't check if already there!
    pub(crate) fn internal_add_fpo(&mut self, fpo: &FixedPriceOffering) {
        self.fpos_by_id.insert(&fpo.offering_id, &fpo);
        self.internal_add_fpo_to_offeror(&fpo.offeror_id, &fpo.offering_id);
    }

    // removes all FPO-related records from Marketplace without initiating any NEAR transfers
    pub(crate) fn internal_remove_fpo(&mut self, offering_id: &OfferingId) -> FixedPriceOffering {
        let removed_fpo = self
            .fpos_by_id
            .remove(offering_id)
            .expect("Could not remove FPO: Could not find FPO listing");
        let offeror_id = &removed_fpo.offeror_id;
        let fpos_by_this_offeror = &mut self
            .fpos_by_offeror_id
            .get(offeror_id)
            .expect("Could not remove FPO: Could not find listings for offeror");
        let did_remove = fpos_by_this_offeror.remove(offering_id);
        assert!(
            did_remove,
            "Could not remove FPO: Offering not on offeror's list"
        );
        if fpos_by_this_offeror.is_empty() {
            self.fpos_by_offeror_id
                .remove(offeror_id)
                .expect("Could not remove FPO: Could not remove the now-empty offeror list");
        }

        removed_fpo
    }

    // add FPO to the set of fpos an offeror offered
    // doesn't check if already there
    pub(crate) fn internal_add_fpo_to_offeror(
        &mut self,
        offeror_id: &AccountId,
        offering_id: &OfferingId,
    ) {
        //get the set of FPOs for the given owner account
        let mut fpo_set = self.fpos_by_offeror_id.get(offeror_id).unwrap_or_else(|| {
            //if the offeror doesn't have any fpos yet we'll create the new unordered set
            UnorderedSet::new(
                MarketplaceStorageKey::FposByOfferorIdInner {
                    account_id_hash: hash_account_id(&offeror_id), // generate a new unique prefix for the collection
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        // insert the nft_account_id into the set
        fpo_set.insert(offering_id);

        // insert back
        self.fpos_by_offeror_id.insert(offeror_id, &fpo_set);
    }

    pub(crate) fn internal_nft_shared_contract_id(&mut self) -> AccountId {
        AccountId::new_unchecked(format!("nft.{}", env::current_account_id()))
    }

    pub(crate) fn fees_account_id(&mut self) -> AccountId {
        AccountId::new_unchecked(format!("fees.{}", env::current_account_id()))
    }
}
