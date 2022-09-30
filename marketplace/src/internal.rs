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
    // doesn't check if already there!
    pub(crate) fn internal_add_primary_listing(&mut self, listing: &PrimaryListing) {
        self.primary_listings_by_id.insert(&listing.id, &listing);
        self.internal_add_primary_listing_to_seller(&listing.seller_id, &listing.id);
    }

    // removes all FPO-related records from Marketplace without initiating any NEAR transfers
    pub(crate) fn internal_remove_primary_listing(&mut self, listing_id: &PrimaryListingId) -> PrimaryListing {
        let removed_listing = self
            .primary_listings_by_id
            .remove(listing_id)
            .expect("Could not remove listing: Could not find listing");
        let seller_id = &removed_listing.seller_id;
        
        let mut listings_by_this_seller = self
            .primary_listings_by_seller_id
            .get(seller_id)
            .expect("Could not remove listing: Could not find listings for this seller");
        let did_remove = listings_by_this_seller.remove(listing_id);
        assert!(
            did_remove,
            "Could not remove listing: Offering not on offeror's list"
        );

        if listings_by_this_seller.is_empty() {
            self.primary_listings_by_seller_id
                .remove(seller_id)
                .expect("Could not remove listing: Could not remove the now-empty seller list");
        } else {
            self.primary_listings_by_seller_id.insert(seller_id, &listings_by_this_seller);
        }

        removed_listing
    }

    // add primary listing to the set of fpos an seller offered
    // doesn't check if already there
    pub(crate) fn internal_add_primary_listing_to_seller(
        &mut self,
        seller_id: &AccountId,
        listing_id: &PrimaryListingId,
    ) {
        //get the set of listings for the given owner account
        let mut listing_set = self.primary_listings_by_seller_id.get(seller_id).unwrap_or_else(|| {
            //if the offeror doesn't have any fpos yet we'll create the new unordered set
            UnorderedSet::new(
                MarketplaceStorageKey::PrimaryListingsBySellerIdInner {
                    account_id_hash: hash_account_id(&seller_id), // generate a new unique prefix for the collection
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        // insert the nft_account_id into the set
        listing_set.insert(listing_id);

        // insert back
        self.primary_listings_by_seller_id.insert(seller_id, &listing_set);
    }

    pub(crate) fn internal_nft_shared_contract_id(&mut self) -> AccountId {
        AccountId::new_unchecked(format!("nft.{}", env::current_account_id()))
    }

    pub(crate) fn fees_account_id(&mut self) -> AccountId {
        AccountId::new_unchecked(format!("fees.{}", env::current_account_id()))
    }
}
