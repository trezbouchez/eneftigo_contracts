use crate::callback::*;
use crate::config::*;
use crate::fpo::config::*;
use crate::fpo::resolve::*;
use crate::FixedPriceOfferingStatus::*;
use crate::*;

// This is required so that the unit tests (placed in separate file) see this
#[cfg(test)]
#[path = "internal_tests.rs"]
mod internal_tests;

impl FixedPriceOffering {
    pub(crate) fn update_status(&mut self) {
        let block_timestamp = env::block_timestamp() as i64;

        if self.status == Ended {
            return;
        }

        if self.supply_left == 0 {
            self.status = Ended;
            return;
        }

        if let Some(end_timestamp) = self.end_timestamp {
            if block_timestamp >= end_timestamp {
                self.status = Ended;
                return;
            }
        }

        if self.status == Running {
            return;
        }

        if let Some(start_timestamp) = self.start_timestamp {
            if block_timestamp >= start_timestamp {
                self.status = Running;
                return;
            }
        } else {
            self.status = Running;
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
        let storage_byte_cost = env::storage_byte_cost();
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

impl FixedPriceOfferingProposal {
    // pub fn mark_unacceptable_and_refund_deposit(&mut self) {
    //     self.is_acceptable = false;
    //     self.refund_deposit();
    // }

    // pub fn refund_deposit(&self) {
    //     Promise::new(self.proposer_id.clone()).transfer(self.price_yocto);
    // }
}

impl MarketplaceContract {
    // will initiate a cross contract call to the nft contract
    // to mint the token and deposit it to the buyer's account
    // if succeeds, Near gets transfered to the seller
    // it will NEITHER (!) decrement the supply NOR close the offering
    /*    pub fn fpo_process_purchase(
        &mut self,
        offering_id: OfferingId,
        buyer_id: AccountId,
        price_yocto: Balance,
        deposit: Balance,
    ) {
        nft_contract::mint(
            offering_id.collection_id,
            buyer_id,
            None, // perpetual royalties
            offering_id.nft_contract_id.clone(),
            deposit, // should be >= 7_060_000_000_000_000_000_000 yN
            NFT_MINT_GAS,
        )
        .then(ext_self_nft::mint_completion(
            offering_id,
            env::current_account_id(), // we are invoking this function on the current contract
            NO_DEPOSIT,                // don't attach any deposit
            NFT_MINT_COMPLETION_GAS,   // GAS attached to the completion call
        ));
    }*/
}
