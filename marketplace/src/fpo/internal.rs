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
        acceptable_proposals_vec_sorted.sort_by(|proposal_a_id,proposal_b_id| {
            let proposal_a = self.proposals.get(proposal_a_id).expect("Could not find proposal");
            let proposal_b = self.proposals.get(proposal_b_id).expect("Could not find proposal");
            proposal_a.cmp(&proposal_b)
        });
        self.acceptable_proposals.clear();
        self.acceptable_proposals.extend(acceptable_proposals_vec_sorted);
    }

    pub(crate) fn is_proposal_acceptable(&self, proposal_id: ProposalId) -> bool {
        for acceptable_proposal_id in self.acceptable_proposals.iter() {
            if acceptable_proposal_id == proposal_id {
                return true
            }
        }
        false
    }

    pub(crate) fn prune_supply_exceeding_acceptable_proposals(&mut self) {
        // assumes acceptable_proposals are already sorted
        let mut acceptable_proposals_vec = self.acceptable_proposals.to_vec();
        let supply_left = self.supply_left as usize;
        if supply_left >= acceptable_proposals_vec.len() {
            return;
        }
        let to_be_pruned_count = acceptable_proposals_vec.len() - supply_left;        
        let pruned_proposals_iter = acceptable_proposals_vec.drain(0..to_be_pruned_count);
        for pruned_proposal_id in pruned_proposals_iter {
            let mut pruned_proposal = self.proposals.get(&pruned_proposal_id)
                .expect("Proposal to be pruned is missing, inconsistent state");
            pruned_proposal.mark_unacceptable_and_refund_deposit();
            self.proposals.insert(&pruned_proposal_id, &pruned_proposal);
        }
        self.acceptable_proposals.clear();
        self.acceptable_proposals.extend(acceptable_proposals_vec);
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
            let worst_acceptable_proposal = self.proposals.get(&worst_acceptable_proposal_id).expect("Could not find proposal");
            worst_acceptable_proposal.price_yocto + PRICE_STEP_YOCTO
        };
    }
}

impl FixedPriceOfferingProposal {

    pub fn mark_unacceptable_and_refund_deposit(&mut self) {
        self.is_acceptable = false;
        self.refund_deposit();
    }

    pub fn refund_deposit(&self) {
        Promise::new(self.proposer_id.clone()).transfer(self.price_yocto);
    }
}

impl MarketplaceContract {

        // will initiate a cross contract call to the nft contract
        // to mint the token and transfer it to the buyer
        // if succeeds, Near will be transfered to the seller
        pub fn fpo_process_purchase(
        &mut self,
        offering_id: OfferingId,
        buyer_id: AccountId,
        price_yocto: Balance
    ) -> Promise {

        // // TODO:
        // let nft_metadata = TokenMetadata {
        //     title: Some("test".to_string()),
        //     description: None,
        //     media: None,
        //     media_hash: None,
        //     copies: Some(1),
        //     issued_at: None,
        //     expires_at: None,
        //     starts_at: None,
        //     updated_at: None,
        //     extra: None,
        //     reference: None,
        //     reference_hash: None,
        // };

        nft_contract::mint(
            offering_id.collection_id,
            buyer_id,
            None, // TODO: setup perpetual royalties
            offering_id.nft_contract_id.clone(),
            1,
            GAS_FOR_NFT_MINT,
        )
        .then(ext_self::fpo_resolve_purchase(
            offering_id,
            price_yocto,
            env::current_account_id(), // we are invoking this function on the current contract
            NO_DEPOSIT,                // don't attach any deposit
            GAS_FOR_NFT_MINT,          // GAS attached to the mint call
        ))
    }
}
