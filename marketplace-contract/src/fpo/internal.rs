use crate::*;
use crate::config::*;
use crate::fpo::config::*;

impl FixedPriceOffering {

    pub(crate) fn sort_winning_proposals(
        &mut self,
    ) {
        let mut winning_proposals_vec_sorted = self.winning_proposals.to_vec();
        winning_proposals_vec_sorted.sort();
        self.winning_proposals.clear();
        self.winning_proposals.extend(winning_proposals_vec_sorted);
    }

    pub(crate) fn prune_supply_exceeding_winning_proposals(
        &mut self,
    ) {
        // assumes winning_proposals are already sorted
        let mut winning_proposals_vec = self.winning_proposals.to_vec();
        let to_be_pruned_count = winning_proposals_vec.len() - self.supply_left as usize;
        let pruned_proposals_iter = winning_proposals_vec.drain(0..to_be_pruned_count);
        for pruned_proposal in pruned_proposals_iter {
            let pruned_proposal_id = pruned_proposal.id;
            let pruned_proposal = &mut self.proposals.get(&pruned_proposal_id).expect("Proposal to be pruned is missing, inconsistent state");
            pruned_proposal.mark_loosing_and_refund_deposit();
        }
        self.winning_proposals.clear();
        self.winning_proposals.extend(winning_proposals_vec);
    }
}

impl FixedPriceOfferingProposal {

    pub(crate) fn mark_loosing_and_refund_deposit(
        &mut self,
    ) {
        self.is_winning = false;
        let deposit_yocto = self.price_yocto * PROPOSAL_DEPOSIT_RATE / 100;
        Promise::new(self.proposer_id.clone()).transfer(deposit_yocto);
    }
}

impl MarketplaceContract {

    pub(crate) fn fpo_process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        nft_token_id: String, 
        buyer_id: AccountId,
    ) -> Promise {
        // initiate a cross contract call to the nft contract. This will mint the token and transfer it to the buyer

        // TODO:
        let nft_metadata = TokenMetadata {
            title: Some("test".to_string()),
            description: None,
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None
        };
        
        ext_contract::nft_mint(
            nft_token_id,
            nft_metadata,
            buyer_id,
            None,       // TODO: setup perpetual royalties
            nft_contract_id.clone(),
            1,
            GAS_FOR_NFT_MINT,
        )
        .then(
            ext_self::fpo_resolve_purchase(
                nft_contract_id,
                env::current_account_id(), // we are invoking this function on the current contract
                NO_DEPOSIT, // don't attach any deposit
                GAS_FOR_NFT_MINT, // GAS attached to the mint call
            )
        )
    }

    // here the caller will need to cover the refund transfers gas if there's supply left
    // this is because there may be multiple winning proposals pending which have active deposits
    // they need to be returned
    // must be called by the offeror!
    pub(crate) fn fpo_conclude(
        &mut self,
        nft_contract_id: AccountId,
    ) {
        // TODO: check if not too early
        // TODO: refund winning offers

        let removed_fpo = self.fpos_by_contract_id.remove(&nft_contract_id).expect("Could not find this NFT listing");

        let offeror_id = removed_fpo.offeror_id;

        // check if the caller can end the listing
        // the marketplace account can end at any time
        // the offeror can end provided the end time has passed or is not set
        let signer_id = env::signer_account_id();
        let contract_id = env::current_account_id();

        if signer_id != contract_id {
            assert!(
                signer_id == offeror_id,
                "Only offeror can conclude the offering."
            );
            if let Some(end_timestamp) = removed_fpo.end_timestamp {
                let current_block_timestamp_nanos = env::block_timestamp() as i64;
                assert!(
                    current_block_timestamp_nanos >= end_timestamp,
                    "Can only conclude after end date has passed"
                );
            }
        }

        let fpos_by_this_offeror = &mut self.fpos_by_offeror_id.get(&offeror_id).expect("Could not find offers for this offeror");
        let did_remove = fpos_by_this_offeror.remove(&nft_contract_id);
        assert!(
            did_remove,
            "Offer not on offeror's list"
        );
        if fpos_by_this_offeror.is_empty() {
            self.fpos_by_offeror_id.remove(&offeror_id).expect("Could not remove the now-empty offer list");
        }
    }
}