use crate::{
    *,
    listing::{
        constants::*,
        status::{ListingStatus},
    },
};

pub(crate) fn hash_primary_listing_id(listing_id: &PrimaryListingId) -> CryptoHash {
    //get the default hash
    //we hash the account ID and return it
    let hashed_string = format!("{}.{}", listing_id.nft_contract_id, listing_id.collection_id);
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

        if let Some(start_timestamp) = self.start_timestamp {
            if block_timestamp >= start_timestamp {
                self.status = ListingStatus::Running;
                return;
            }
        } else {
            self.status = ListingStatus::Running;
        }
    }

    pub(crate) fn sort_proposals(&mut self) {
        let mut proposals_vec_sorted = self.proposals.to_vec();
        proposals_vec_sorted.sort();
        self.proposals.clear();
        self.proposals.extend(proposals_vec_sorted);
    }

    // pub(crate) fn is_proposal_acceptable(&self, proposal_id: ProposalId) -> bool {
    //     for acceptable_proposal_id in self.acceptable_proposals.iter() {
    //         if acceptable_proposal_id == proposal_id {
    //             return true;
    //         }
    //     }
    //     false
    // }

    pub(crate) fn remove_supply_exceeding_proposals_and_refund_proposers(&mut self) {
        if self.supply_left >= self.proposals.len() {
            return;
        }
        let num_outbid_proposals = self.proposals.len() - self.supply_left;
        for _ in 0..num_outbid_proposals {
            let storage_before = env::storage_usage();
            let removed_proposal = self
                .proposals
                .pop()
                .expect("Could not remove proposal. acceptable_proposals is empty");
            let storage_after = env::storage_usage();
            let freed_storage = storage_before - storage_after;
            let freed_storage_cost = freed_storage as Balance * env::storage_byte_cost();
            let refund = removed_proposal.price_yocto + freed_storage_cost;
            Promise::new(removed_proposal.proposer_id).transfer(refund);
        }
    }

    pub(crate) fn acceptable_price_yocto(&self) -> u128 {
        assert!(
            self.min_proposal_price_yocto.is_some(),
            "This offer does not accept proposals"
        );
        let num_proposals = self.proposals.len();
        let unmatched_supply_exists = num_proposals < self.supply_left;
        return if unmatched_supply_exists {
            self.min_proposal_price_yocto.unwrap()
        } else {
            let worst_acceptable_proposal = self.proposals.get(num_proposals - 1).unwrap();
            worst_acceptable_proposal.price_yocto + PRICE_STEP_YOCTO
        };
    }
}
