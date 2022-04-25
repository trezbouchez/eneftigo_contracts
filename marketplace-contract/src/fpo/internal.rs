use crate::*;

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
            let mut pruned_proposal = self.proposals.get(&pruned_proposal_id).expect("Proposal to be pruned is missing, inconsistent state");
            pruned_proposal.is_winning = false;
        }
        self.winning_proposals.clear();
        self.winning_proposals.extend(winning_proposals_vec);
    }

    pub(crate) fn process_purchase(
        &mut self,
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
            self.supply_left.to_string(),
            nft_metadata,
            buyer_id.clone(),
            None,       // TODO: setup perpetual royalties
            self.nft_contract_id.clone(),
            1,
            GAS_FOR_NFT_MINT,
        )
        .then(
            ext_self::fpo_resolve_purchase(
                self.nft_contract_id.clone(),
                buyer_id,
                env::current_account_id(), // we are invoking this function on the current contract
                NO_DEPOSIT, // don't attach any deposit
                GAS_FOR_NFT_MINT, // GAS attached to the mint call
            )
        )
    }
}