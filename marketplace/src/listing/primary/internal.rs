use crate::{
    internal::hash_account_id,
    listing::{constants::*, status::ListingStatus},
    *,
};

pub(crate) fn hash_primary_listing_id(listing_id: &PrimaryListingId) -> CryptoHash {
    let hashed_string = format!(
        "{}.{}",
        listing_id.nft_contract_id, listing_id.collection_id
    );
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(hashed_string.as_bytes()));
    hash
}

// This is required so that the unit tests (placed in separate file) see this
#[cfg(test)]
#[path = "internal_tests.rs"]
mod internal_tests;

impl PrimaryListing {
    pub(crate) fn update_status(&mut self) {
        let block_timestamp = env::block_timestamp() as i64;

        if self.status == ListingStatus::Ended {
            return;
        }

        if self.supply_left == 0 {
            self.status = ListingStatus::Ended;
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

    pub(crate) fn sort_bids(&mut self) {
        let mut bids_vec_sorted = self.bids.to_vec();
        bids_vec_sorted.sort();
        self.bids.clear();
        self.bids.extend(bids_vec_sorted);
    }

    pub(crate) fn acceptable_bid_yocto(&self) -> u128 {
        let min_bid_yocto = self.min_bid_yocto.expect("This offer does not accept bids");
        let num_bids = self.bids.len();
        let unmatched_supply_exists = num_bids < self.supply_left;
        return if unmatched_supply_exists {
            min_bid_yocto
        } else {
            let worst_acceptable_bid = self.bids.get(num_bids - 1).unwrap();
            worst_acceptable_bid.amount_yocto + BID_STEP_YOCTO
        };
    }
}

impl MarketplaceContract {
    // doesn't check if already there!
    pub(crate) fn internal_add_primary_listing(&mut self, listing: &PrimaryListing) {
        self.primary_listings_by_id.insert(&listing.id, &listing);
        self.internal_add_primary_listing_to_seller(&listing.seller_id, &listing.id);
    }

    // removes all FPO-related records from Marketplace without initiating any NEAR transfers
    pub(crate) fn internal_remove_primary_listing(
        &mut self,
        listing_id: &PrimaryListingId,
    ) -> PrimaryListing {
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
            self.primary_listings_by_seller_id
                .insert(seller_id, &listings_by_this_seller);
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
        let mut listing_set = self
            .primary_listings_by_seller_id
            .get(seller_id)
            .unwrap_or_else(|| {
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
        self.primary_listings_by_seller_id
            .insert(seller_id, &listing_set);
    }

    // this won't insert updated listing back into contract, caller must do it (if needed)
    pub(crate) fn primary_listing_remove_supply_exceeding_bids_and_refund_bidders(
        &mut self,
        listing: &mut PrimaryListing,
    ) {
        // TODO: this won't work with SecondaryListing reference! will lead to inconsistent state
        if listing.supply_left >= listing.bids.len() {
            return;
        }
        let num_outbid_bids = listing.bids.len() - listing.supply_left;
        for _ in 0..num_outbid_bids {
            let storage_before = env::storage_usage();
            let removed_bid = listing
                .bids
                .pop()
                .expect("Could not remove a bid");
            let bidder_id = removed_bid.bidder_id;
            Promise::new(bidder_id.clone()).transfer(removed_bid.amount_yocto);
            let storage_after = env::storage_usage();
            let freed_storage = storage_before - storage_after; // this was covered by bidder
            let freed_storage_cost = freed_storage as Balance * env::storage_byte_cost();
            if let Some(current_deposit) = self.storage_deposits.get(&bidder_id) {
                let updated_deposit = current_deposit + freed_storage_cost;
                self.storage_deposits.insert(&bidder_id, &updated_deposit);
            } else {
                // this should never happen! TODO: we may want to log some message if it does
            }
        }
    }
}
