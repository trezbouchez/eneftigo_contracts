use crate::{
    *,
    internal::{hash_account_id},
    listing::{status::ListingStatus, secondary::lib::SecondaryListingId},
};

pub(crate) fn hash_secondary_listing_id(listing_id: &SecondaryListingId) -> CryptoHash {
    let hashed_string = format!("{}.{}", listing_id.nft_contract_id, listing_id.token_id);
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(hashed_string.as_bytes()));
    hash
}

impl MarketplaceContract {
    // doesn't check if already there!
    pub(crate) fn internal_add_secondary_listing(&mut self, listing: &SecondaryListing) {
        self.secondary_listings_by_id.insert(&listing.id, &listing);
        self.internal_add_secondary_listing_to_seller(&listing.seller_id, &listing.id);
    }

    // removes all FPO-related records from Marketplace without initiating any NEAR transfers
    pub(crate) fn internal_remove_secondary_listing(
        &mut self,
        listing_id: &SecondaryListingId,
    ) -> SecondaryListing {
        let removed_listing = self
            .secondary_listings_by_id
            .remove(listing_id)
            .expect("Could not remove listing: Could not find listing");
        let seller_id = &removed_listing.seller_id;

        let mut listings_by_this_seller = self
            .secondary_listings_by_seller_id
            .get(seller_id)
            .expect("Could not remove listing: Could not find listings for this seller");
        let did_remove = listings_by_this_seller.remove(listing_id);
        assert!(
            did_remove,
            "Could not remove listing: Offering not on offeror's list"
        );

        if listings_by_this_seller.is_empty() {
            self.secondary_listings_by_seller_id
                .remove(seller_id)
                .expect("Could not remove listing: Could not remove the now-empty seller list");
        } else {
            self.secondary_listings_by_seller_id
                .insert(seller_id, &listings_by_this_seller);
        }

        removed_listing
    }

    // add seconday listing to the set of fpos an seller offered
    // doesn't check if already there
    pub(crate) fn internal_add_secondary_listing_to_seller(
        &mut self,
        seller_id: &AccountId,
        listing_id: &SecondaryListingId,
    ) {
        //get the set of listings for the given owner account
        let mut listing_set = self
            .secondary_listings_by_seller_id
            .get(seller_id)
            .unwrap_or_else(|| {
                //if the offeror doesn't have any fpos yet we'll create the new unordered set
                UnorderedSet::new(
                    MarketplaceStorageKey::SecondaryListingsBySellerIdInner {
                        account_id_hash: hash_account_id(&seller_id), // generate a new unique prefix for the collection
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });

        // insert the nft_account_id into the set
        listing_set.insert(listing_id);

        // insert back
        self.secondary_listings_by_seller_id
            .insert(seller_id, &listing_set);
    }
}

impl SecondaryListing {
    pub(crate) fn update_status(&mut self) {
        let block_timestamp = env::block_timestamp() as i64;

        if self.status == ListingStatus::Ended {
            return;
        }

        if let Some(end_timestamp) = self.end_timestamp {
            if block_timestamp >= end_timestamp {
                self.status = ListingStatus::Ended;
                return;
            }
        }

        if self.status == ListingStatus::Running {
            return;
        }

        if block_timestamp >= self.start_timestamp {
            self.status = ListingStatus::Running;
            return;
        }
    }
}