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

    pub(crate) fn sort_acceptable_proposals(&mut self) {
        let mut acceptable_proposals_vec_sorted = self.acceptable_proposals.to_vec();
        acceptable_proposals_vec_sorted.sort_by(|proposal_a_id, proposal_b_id| {
            let proposal_a = self
                .proposals
                .get(proposal_a_id)
                .expect("Could not find proposal");
            let proposal_b = self
                .proposals
                .get(proposal_b_id)
                .expect("Could not find proposal");
            proposal_a.cmp(&proposal_b)
        });
        self.acceptable_proposals.clear();
        self.acceptable_proposals
            .extend(acceptable_proposals_vec_sorted);
    }

    pub(crate) fn is_proposal_acceptable(&self, proposal_id: ProposalId) -> bool {
        for acceptable_proposal_id in self.acceptable_proposals.iter() {
            if acceptable_proposal_id == proposal_id {
                return true;
            }
        }
        false
    }

    pub(crate) fn prune_supply_exceeding_acceptable_proposals_and_refund_proposers(&mut self) {
        // assumes acceptable_proposals are already sorted
        let storage_byte_cost = env::storage_byte_cost();
        while self.acceptable_proposals.len() > self.supply_left {
            let storage_before_proposal_was_pruned = env::storage_usage();
            let pruned_proposal_id = self.acceptable_proposals.swap_remove(0);
            self.acceptable_proposals.pop();
            let mut pruned_proposal = self
                .proposals
                .get(&pruned_proposal_id)
                .expect("Proposal to be pruned is missing, inconsistent state");
            pruned_proposal.is_acceptable = false;
            self.proposals.insert(&pruned_proposal_id, &pruned_proposal);
            let storage_after_proposal_was_pruned = env::storage_usage();
            let storage_freed =
                storage_before_proposal_was_pruned - storage_after_proposal_was_pruned;
            let storage_freed_cost = storage_freed as Balance * storage_byte_cost;
            let refund = pruned_proposal.price_yocto + storage_freed_cost;
            // let str = format!(
            //     "Loosing proposal refund: {}, price: {}, freed storage: {}",
            //     refund, pruned_proposal.price_yocto, storage_freed_cost
            // );
            // env::log_str(&str);
            if refund > 0 {
                Promise::new(pruned_proposal.proposer_id).transfer(refund);
            }
        }
    }

    pub(crate) fn acceptable_price_yocto(&self) -> u128 {
        assert!(
            self.min_proposal_price_yocto.is_some(),
            "This offer does not accept proposals"
        );
        let unmatched_supply_exists = self.acceptable_proposals.len() < self.supply_left;
        return if unmatched_supply_exists {
            self.min_proposal_price_yocto.unwrap()
        } else {
            let worst_acceptable_proposal_id = self.acceptable_proposals.get(0).unwrap();
            let worst_acceptable_proposal = self
                .proposals
                .get(&worst_acceptable_proposal_id)
                .expect("Could not find proposal");
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
